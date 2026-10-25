pub use shall::app::LuaHooks;
pub use shall::backends::generic::{
    GenericBackendCore, GenericInstallable, GenericQueryable, GenericSearchable, GenericUpgradable,
    ManagerConfig, ManualListing, RepoListing, SearchSource,
};
use shall::backends::registry::language::*;
use shall::backends::registry::os_native::*;
use shall::backends::registry::system::*;
pub use shall::backends::registry::*;
pub use shall::config::Config;
pub use shall::core::{BackendCapabilities, CommandExecutor};
pub use shall::parsers::LambdaParser;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::trace;
use Expect::{NoCommand, Runs, RunsOneOf, Unsupported};

macro_rules! rows_as_registrars {
    ($($fn_name:ident => $row:literal),* $(,)?) => {
        $(
            fn $fn_name(reg: &mut BackendRegistry, exec: &CommandExecutor) {
                shall::backends::onboarder::register_builtin_row(reg, exec, $row);
            }
        )*
    };
}

rows_as_registrars! {
    register_asdf => "asdf", register_cabal => "cabal", register_cargo => "cargo",
    register_composer => "composer", register_emerge => "emerge", register_eopkg => "eopkg",
    register_guix => "guix", register_helm => "helm", register_krew => "krew",
    register_luarocks => "luarocks", register_moss => "moss", register_mix => "mix",
    register_nimble => "nimble", register_npm => "npm", register_opam => "opam",
    register_pipx => "pipx", register_pixi => "pixi", register_pnpm => "pnpm",
    register_pubdart => "pub", register_slackpkg => "slackpkg", register_spack => "spack",
    register_stack => "stack", register_uv => "uv", register_yarn => "yarn",
}

async fn removal_token() -> shall::app::sync::guard::Reaped {
    crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await
}

async fn build_registry() -> BackendRegistry {
    let exec = CommandExecutor::new(true, false);
    let config = Config::default();
    let hooks = Arc::new(LuaHooks::new(&config).expect("hooks init"));
    create_default_registry(exec, &config, hooks).await
}
fn caps(b: &BackendCapabilities) -> Vec<&'static str> {
    let mut v = Vec::new();
    if b.as_installable().is_some() {
        v.push("installable");
    }
    if b.as_queryable().is_some() {
        v.push("queryable");
    }
    if b.as_searchable().is_some() {
        v.push("searchable");
    }
    if b.as_upgradable().is_some() {
        v.push("upgradable");
    }
    if b.as_repo_manager().is_some() {
        v.push("repo_manager");
    }
    if b.as_metadata_provider().is_some() {
        v.push("metadata_provider");
    }
    v
}
fn assert_caps(reg: &BackendRegistry, name: &str, expected: &[&str]) {
    let b = reg
        .get(name)
        .unwrap_or_else(|| panic!("backend '{}' is not registered", name));
    let got = caps(&b);
    for cap in expected {
        assert!(
            got.contains(cap),
            "backend '{}' is missing capability '{}' (has {:?})",
            name,
            cap,
            got
        );
    }
    assert_eq!(
        got.len(),
        expected.len(),
        "backend '{}' capability set mismatch: got {:?}, expected {:?}",
        name,
        got,
        expected
    );
}
enum Expect {
    /// A call containing this substring must have run.
    Runs(&'static str),
    /// One of the host's detected provider commands must have run.
    RunsOneOf(&'static [&'static str]),
    /// The verb must refuse with `Unsupported` and run **nothing**. Asserted rather than
    /// skipped: a manager with no uninstall verb that silently ran *something* would leave
    /// the model claiming a package is gone that is still installed.
    Unsupported,
    /// This verb runs no command at all, and the reason. A download backend fetches over
    /// HTTP and a link backend writes a symlink; neither shells out, so "no argv" is the
    /// correct answer rather than a gap. The reason is the exemption — an unexplained one
    /// is a backend nobody looked at wearing the costume of one somebody did (E29).
    NoCommand(&'static str),
}
struct ArgvCase {
    backend: &'static str,
    /// A closure, not a function pointer: the registrars in this file take
    /// `(reg, exec)` and every module-owned one also takes a `&Config`. A row that is a
    /// closure adapts in place, so adding a backend does not also add a wrapper function
    /// nobody reads — `register_psresource_for_test` was the first of twenty-eight.
    register: &'static dyn Fn(&mut BackendRegistry, &CommandExecutor),
    /// What the declaration names. `jq` for a package manager — but `setting:` addresses
    /// `SUBKEY/VALUE` and `lvm:` addresses `group/volume`, and a package name is neither.
    /// Driving every backend with `"jq"` would have tested those backends' *refusals* and
    /// reported it as argv coverage.
    subject: &'static str,
    options: &'static [(&'static str, &'static str)],
    install: Expect,
    remove: Expect,
    /// Answers injected before the *removal* half runs: the ask-first effectors (zfs/lvm)
    /// need "the object is present", or they converge on absent and run nothing. The
    /// install half keeps the mock default, which reads as absent — exactly what an
    /// install's own ask wants. Empty means no stubs.
    remove_stubs: &'static [(&'static str, &'static str)],
}
impl ArgvCase {
    /// A package manager: the declaration is a bare package name.
    fn pkg(
        backend: &'static str,
        register: &'static dyn Fn(&mut BackendRegistry, &CommandExecutor),
        install: Expect,
        remove: Expect,
    ) -> Self {
        Self {
            backend,
            register,
            subject: "jq",
            options: &[],
            install,
            remove,
            remove_stubs: &[],
        }
    }
    /// A backend whose declaration is not a package name.
    fn shaped(
        backend: &'static str,
        register: &'static dyn Fn(&mut BackendRegistry, &CommandExecutor),
        subject: &'static str,
        options: &'static [(&'static str, &'static str)],
        install: Expect,
        remove: Expect,
    ) -> Self {
        Self {
            backend,
            register,
            subject,
            options,
            install,
            remove,
            remove_stubs: &[],
        }
    }
    /// Attach the existence answers this backend's removal asks for.
    fn with_remove_stubs(mut self, stubs: &'static [(&'static str, &'static str)]) -> Self {
        self.remove_stubs = stubs;
        self
    }
}
fn argv_cases() -> Vec<ArgvCase> {
    // `mut` is used by the BSD-target pushes at the end; other targets never touch it.
    #[allow(unused_mut)]
    let mut cases = vec![
        // ---- OS-native system managers, each invisible to every platform's CI but its own.
        ArgvCase::pkg(
            "apt",
            &register_apt,
            Runs("apt install -y -- jq"),
            Runs("apt remove -y -- jq"),
        ),
        ArgvCase::pkg(
            "apk",
            &register_apk,
            Runs("apk add -- jq"),
            Runs("apk del -- jq"),
        ),
        ArgvCase::pkg(
            "zypper",
            &register_zypper,
            Runs("zypper install -y"),
            Runs("zypper remove -y"),
        ),
        ArgvCase::pkg(
            "winget",
            &register_winget,
            Runs("winget install"),
            Runs("winget uninstall"),
        ),
        ArgvCase::pkg(
            "scoop",
            &register_scoop,
            Runs("scoop install"),
            Runs("scoop uninstall"),
        ),
        ArgvCase::pkg(
            "choco",
            &register_choco,
            Runs("choco install"),
            Runs("choco uninstall"),
        ),
        ArgvCase::pkg(
            "mas",
            &register_mas,
            Runs("mas install"),
            Runs("mas uninstall"),
        ),
        ArgvCase::pkg(
            "macports",
            &register_macports,
            Runs("port install"),
            Runs("port uninstall"),
        ),
        // PowerShell's module manager. Its module was `#[cfg(target_os = "windows")]` until
        // 2026-07-30, so it could not appear in this table at all: the row would not compile
        // where it is most needed, which is every platform that cannot run PSResourceGet.
        ArgvCase::pkg(
            "psresource",
            &|r, e| shall::backends::psresource::register(r, e, &Config::default()),
            Runs("Install-PSResource"),
            Runs("Uninstall-PSResource"),
        ),
        // The two rows that carried the defect as their expectation. `--` is not decoration
        // here: these are the two managers that run as root.
        ArgvCase::pkg(
            "pacman",
            &register_pacman,
            Runs("pacman -S --noconfirm --needed -- jq"),
            Runs("pacman -Rs --noconfirm -- jq"),
        ),
        // No terminator, and that is the fix rather than the omission: `dnf` is dnf5 on
        // Fedora 41+, whose parser answers `Unknown argument "--"`. This expectation
        // demanded the broken form and was green the whole time the installs were failing.
        ArgvCase::pkg(
            "dnf",
            &register_dnf,
            Runs("dnf install -y jq"),
            Runs("dnf remove -y jq"),
        ),
        // Void's manager installs and removes with two different programs, which is the
        // `remove_binary` case a single-binary assumption gets wrong.
        ArgvCase::pkg(
            "xbps",
            &register_xbps,
            Runs("xbps-install -Sy -- jq"),
            Runs("xbps-remove -y -- jq"),
        ),
        ArgvCase::pkg(
            "guix",
            &register_guix,
            Runs("guix install"),
            Runs("guix remove"),
        ),
        ArgvCase::pkg(
            "emerge",
            &register_emerge,
            Runs("emerge"),
            Runs("--unmerge"),
        ),
        ArgvCase::pkg(
            "eopkg",
            &register_eopkg,
            Runs("eopkg install -y"),
            Runs("eopkg remove -y"),
        ),
        ArgvCase::pkg(
            "slackpkg",
            &register_slackpkg,
            Runs("slackpkg -batch=on"),
            Runs("remove"),
        ),
        ArgvCase::pkg(
            "moss",
            &register_moss,
            Runs("moss install"),
            Runs("moss remove"),
        ),
        // The AUR helpers: pacman-syntax, registered on Linux only, and until 2026-07-30
        // reached through a five-argument helper no row could name.
        ArgvCase::pkg("yay", &register_yay, Runs("yay -S"), Runs("yay -Rs")),
        ArgvCase::pkg("paru", &register_paru, Runs("paru -S"), Runs("paru -Rs")),
        // ---- Cross-platform store-shaped managers.
        ArgvCase::pkg(
            "brew",
            &|r, e| shall::backends::brew::register(r, e, &Config::default()),
            Runs("brew install -- jq"),
            Runs("brew uninstall -- jq"),
        ),
        // `snap info` first: the install path asks whether the snap is classic before it
        // installs, so both calls are the argv and asserting only the second would let the
        // probe change without notice.
        ArgvCase::pkg(
            "snap",
            &|r, e| shall::backends::snap::register(r, e, &Config::default()),
            Runs("snap install -- jq"),
            Runs("snap remove -- jq"),
        ),
        ArgvCase::pkg(
            "flatpak",
            &|r, e| shall::backends::flatpak::register(r, e, &Config::default()),
            // `--or-update` because flatpak calls an already-installed ref an error and
            // exits non-zero (`Y23`).
            Runs("flatpak install --system -y --noninteractive --or-update -- jq"),
            Runs("flatpak uninstall --system -y --noninteractive -- jq"),
        ),
        ArgvCase::pkg(
            "nix",
            &|r, e| shall::backends::nix::register(r, e, &Config::default()),
            Runs("nix profile install -- nixpkgs#jq"),
            // nix removes by index, so it must read the profile before it can name what to
            // remove. The listing IS the removal's first argv; a row asserting a
            // `nix profile remove` that never runs would pin a command that does not exist.
            Runs("nix profile list --json"),
        ),
        ArgvCase::pkg(
            "conda",
            &|r, e| register_conda(r, e, &Config::default()),
            Runs("conda install -n base -y -- jq"),
            Runs("conda remove -n base -y -- jq"),
        ),
        // ---- Language managers.
        ArgvCase::pkg(
            "pip",
            &register_pip,
            Runs("pip install"),
            Runs("pip uninstall"),
        ),
        ArgvCase::pkg(
            "gem",
            &register_gem,
            Runs("gem install"),
            Runs("gem uninstall"),
        ),
        ArgvCase::pkg("bun", &register_bun, Runs("bun add"), Runs("bun remove")),
        ArgvCase::pkg(
            "dotnet",
            &register_dotnet,
            Runs("dotnet tool install"),
            Runs("dotnet tool uninstall"),
        ),
        ArgvCase::pkg(
            "cargo",
            &register_cargo,
            Runs("cargo install -- jq"),
            Runs("cargo uninstall -- jq"),
        ),
        ArgvCase::pkg(
            "pipx",
            &register_pipx,
            Runs("pipx install -- jq"),
            Runs("pipx uninstall -- jq"),
        ),
        ArgvCase::pkg(
            "uv",
            &register_uv,
            Runs("uv tool install -- jq"),
            Runs("uv tool uninstall -- jq"),
        ),
        // The three Node managers spell the same two verbs three ways, which is exactly why
        // each needs its own row: `npm install -g` / `pnpm add -g` / `yarn global add`.
        ArgvCase::pkg(
            "npm",
            &register_npm,
            Runs("npm install -g -- jq"),
            Runs("npm uninstall -g -- jq"),
        ),
        ArgvCase::pkg(
            "pnpm",
            &register_pnpm,
            Runs("pnpm add -g -- jq"),
            Runs("pnpm remove -g -- jq"),
        ),
        ArgvCase::pkg(
            "yarn",
            &register_yarn,
            Runs("yarn global add -- jq"),
            Runs("yarn global remove -- jq"),
        ),
        ArgvCase::pkg(
            "mise",
            &|r, e| shall::backends::mise::register(r, e, &Config::default()),
            Runs("mise use -g -- jq@latest"),
            Runs("mise uninstall -- jq"),
        ),
        // `go install` takes a module path, not a package name, and removal is deleting the
        // binary out of GOPATH/bin — so the only argv removal runs is the question of where
        // that is. Asserting a `go uninstall` would pin a verb the go tool does not have.
        ArgvCase::shaped(
            "go",
            &|r, e| shall::backends::go::register(r, e, &Config::default()),
            "github.com/mikefarah/yq/v4",
            &[],
            Runs("go install -- github.com/mikefarah/yq/v4@latest"),
            Runs("go env GOPATH"),
        ),
        ArgvCase::pkg(
            "pub",
            &register_pubdart,
            Runs("dart pub global activate -- jq"),
            Runs("dart pub global deactivate -- jq"),
        ),
        ArgvCase::pkg(
            "krew",
            &register_krew,
            Runs("kubectl krew install -- jq"),
            Runs("kubectl krew uninstall -- jq"),
        ),
        ArgvCase::pkg(
            "composer",
            &register_composer,
            Runs("composer global require"),
            Runs("global remove"),
        ),
        ArgvCase::pkg(
            "opam",
            &register_opam,
            Runs("opam install"),
            Runs("opam remove"),
        ),
        ArgvCase::pkg(
            "luarocks",
            &register_luarocks,
            Runs("luarocks install"),
            Runs("luarocks remove"),
        ),
        ArgvCase::pkg(
            "nimble",
            &register_nimble,
            Runs("nimble install"),
            Runs("nimble uninstall"),
        ),
        ArgvCase::pkg(
            "pixi",
            &register_pixi,
            Runs("pixi global install"),
            Runs("pixi global uninstall"),
        ),
        ArgvCase::pkg(
            "spack",
            &register_spack,
            Runs("spack install"),
            Runs("spack uninstall"),
        ),
        ArgvCase::pkg(
            "mix",
            &register_mix,
            Runs("mix archive.install"),
            Runs("mix archive.uninstall"),
        ),
        ArgvCase::pkg(
            "asdf",
            &register_asdf,
            Runs("asdf install"),
            Runs("asdf uninstall"),
        ),
        // helm installs from `@url=` and lists/removes by name. It was exempt from this
        // table while a row could only carry a package name — the exemption said the row
        // "would pass on the remove alone", which was true of the table's *shape*, not of
        // helm. A row that carries options covers it, and the exemption is retired.
        ArgvCase::shaped(
            "helm",
            &register_helm,
            "shall-probe",
            &[("url", "https://example.invalid/p.tgz")],
            Runs("helm plugin install -- https://example.invalid/p.tgz"),
            Runs("helm plugin uninstall -- shall-probe"),
        ),
        // The two Haskell managers, which have no uninstall verb at all.
        ArgvCase::pkg("cabal", &register_cabal, Runs("cabal install"), Unsupported),
        ArgvCase::pkg("stack", &register_stack, Runs("stack install"), Unsupported),
        // ---- Editor extension hosts.
        ArgvCase::pkg(
            "vscode",
            &|r, e| shall::backends::vscode::register(r, e, &Config::default()),
            Runs("code --force --install-extension jq"),
            Runs("code --uninstall-extension jq"),
        ),
        // Emacs is handed an Emacs Lisp form, not a subcommand — which is why
        // `argv_drift_tests` excuses it from the `--help` walk. The form's *shape* is still
        // argv and still drifts, so it is asserted here rather than nowhere.
        //
        // The form batches (`Q46`): `(dolist (p '(a b)) (package-install p))` rather than
        // one `(package-install 'a)` per Emacs launch, because each launch also paid for a
        // `package-refresh-contents`. What is pinned is that the *name reaches the form* —
        // this test caught the change when it was made, which is the whole point of it.
        ArgvCase::pkg(
            "emacs",
            &|r, e| shall::backends::emacs::register(r, e, &Config::default()),
            Runs("(dolist (p '(jq)) (package-install p))"),
            Runs("(package-delete p)"),
        ),
        // ---- Resource backends: the declaration is not a package name, and each addresses
        // its own kind of thing. These are the rows the old table's shape could not hold.
        ArgvCase::shaped(
            "lvm",
            &|r, e| shall::backends::storage::register(r, e, &Config::default()),
            "vg0/data",
            &[("size", "1G")],
            Runs("lvcreate -n data -L 1G vg0"),
            Runs("lvremove -y vg0/data"),
        )
        .with_remove_stubs(&[(
            // Removal asks whether the volume is there before it removes (already-absent
            // is convergence); the stub answers "present".
            "lvs --noheadings --units b --nosuffix -o vg_name,lv_name vg0/data",
            "vg0 data\n",
        )]),
        ArgvCase::shaped(
            "zfs",
            &|r, e| shall::backends::storage::register(r, e, &Config::default()),
            "tank/data",
            &[],
            Runs("zfs create tank/data"),
            Runs("zfs destroy -r tank/data"),
        )
        .with_remove_stubs(&[("zfs list -H -o name tank/data", "tank/data\n")]),
        ArgvCase::shaped(
            "btrfs",
            &|r, e| shall::backends::btrfs::register(r, e, &Config::default()),
            "/mnt/shall-probe",
            &[],
            Runs("btrfs subvolume create /mnt/shall-probe"),
            NoCommand(
                "deletion is guarded on the subvolume existing on the real filesystem \
                 (`Path::exists`), which no mock can satisfy. Deleting a path that is not \
                 there is the one case where running nothing is right.",
            ),
        ),
        // `service:` and `setting:` dispatch on which init system / settings store this
        // HOST has, not on which OS the code was compiled for — `sc` here, `systemctl`
        // there. So these two rows assert the provider this host selects, and each platform's
        // CI covers its own. Both backends additionally have provider-table tests that run
        // everywhere.
        ArgvCase::shaped(
            "service",
            &|r, e| shall::backends::service::register(r, e, &Config::default()),
            "nginx",
            &[("state", "running")],
            // **Three platforms, three init systems.** The backend detects which one is
            // driving the host — systemd, launchd, `sc` — so an `else` meaning "Linux" is a
            // guess that holds until somebody builds on a Mac. Nobody had: the matrix
            // produced one target out of four, and the first macOS run in this repo's
            // history reported `launchctl load -w nginx` against an expectation of
            // `systemctl`. The product was right; the test had two branches for three cases.
             RunsOneOf(&["sc start nginx", "launchctl", "systemctl", "rc-service nginx start"]),
             RunsOneOf(&["sc stop nginx", "launchctl", "systemctl", "rc-service nginx stop"]),
        ),
        ArgvCase::shaped(
            "setting",
            &|r, e| shall::backends::setting::register(r, e, &Config::default()),
            if cfg!(windows) {
                "Software\\ShallProbe/Value"
            } else {
                "org.shall.probe/key"
            },
            &[("value", "1")],
            // **macOS has no settings store at all**, and this row is where that becomes a
            // stated fact rather than a surprise. `setting_stores.toml` ships `gsettings`
            // (`os = "linux"`) and the Windows registry, and nothing else — so on a Mac the
            // backend finds no adapter, refuses by name, and runs nothing. That is a real
            // gap in `setting:` coverage and not a defect in this test; recording it here
            // is how the gap stops being invisible.
            if cfg!(target_os = "macos") {
                NoCommand(
                    "macOS ships no `[[setting_store]]` row — `setting_stores.toml` covers \
                     gsettings and the Windows registry — so the backend finds no adapter \
                     and refuses by name rather than running anything",
                )
            } else {
                Runs(if cfg!(windows) {
                    "reg add HKCU\\Software\\ShallProbe /v Value /d 1 /f"
                } else {
                    "gsettings set org.shall.probe key 1"
                })
            },
            if cfg!(target_os = "macos") {
                NoCommand(
                    "the same, from the other side: with no adapter for this machine there \
                     is nothing to reset, and the removal refuses by name having run nothing",
                )
            } else {
                Runs(if cfg!(windows) {
                    "reg delete HKCU\\Software\\ShallProbe /v Value /f"
                } else {
                    "gsettings reset org.shall.probe key"
                })
            },
        ),
        // ---- Backends that run no command. Each fetches over HTTP or writes to the
        // filesystem directly, so "no argv" is the right answer and not a hole — but it is
        // asserted, because a download backend that started shelling out to `curl` would
        // otherwise change from "no calls" to "some calls" with nothing watching.
        ArgvCase::shaped(
            "link",
            &|r, e| shall::backends::link::register(r, e, &Config::default()),
            "/tmp/shall-probe-src",
            &[("target", "/tmp/shall-probe-dst")],
            NoCommand(
                "writes a symlink (or copies) through the filesystem layer. It shells out \
                 for nothing, which is why a link works on a machine with no shell at all.",
            ),
            NoCommand("removes the file it wrote, through the same filesystem layer."),
        ),
        ArgvCase::shaped(
            "dir",
            &|r, e| shall::backends::dir::register(r, e, &Config::default()),
            "/tmp/shall-probe-dir",
            &[],
            NoCommand(
                "creates a directory through the filesystem layer. It runs no package-manager command, \
                 so there is no argv to check.",
            ),
            NoCommand("removes an empty directory through the filesystem layer."),
        ),
        ArgvCase::shaped(
            "web",
            &|r, e| shall::backends::web::register(r, e, &Config::default()),
            "https://example.invalid/probe.tar.gz",
            &[("unverified", "true")],
            NoCommand(
                "fetches over HTTP and writes the file itself. The scheme and checksum \
                 refusals happen before any process could be started.",
            ),
            NoCommand("deletes the file it downloaded; no process is involved."),
        ),
        ArgvCase::pkg(
            "github",
            &|r, e| shall::backends::github::register(r, e, &Config::default()),
            NoCommand(
                "resolves a release through the GitHub API and downloads the asset over \
                 HTTP. Asset SELECTION is the thing worth pinning here and has its own \
                 tests and a recorded lock; no argv is involved in either half.",
            ),
            NoCommand("deletes the extracted artifact; no process is involved."),
        ),
        ArgvCase::shaped(
            "appimage",
            &|r, e| shall::backends::appimage::register(r, e, &Config::default()),
            "https://example.invalid/probe.AppImage",
            &[("unverified", "true")],
            NoCommand(
                "downloads the image over HTTP and marks it executable through the \
                 filesystem layer — the same shape as `web`, one file format along.",
            ),
            NoCommand("deletes the image it downloaded; no process is involved."),
        ),
    ];
    // The BSD tools, where removal is a different program — pushed rather than listed,
    // because their registrars are compiled only where each packaging universe lives
    // (same gating as the registration site).
    #[cfg(target_os = "netbsd")]
    cases.push(ArgvCase::pkg(
        "pkgin",
        &register_pkgin,
        Runs("pkgin -y install"),
        Runs("pkgin -y remove"),
    ));
    #[cfg(target_os = "freebsd")]
    cases.push(ArgvCase::pkg(
        "pkg",
        &register_pkg_freebsd,
        Runs("pkg install -y"),
        Runs("pkg delete -y"),
    ));
    #[cfg(target_os = "openbsd")]
    cases.push(ArgvCase::pkg(
        "pkg_add",
        &register_pkg_add_openbsd,
        Runs("pkg_add"),
        Runs("pkg_delete"),
    ));
    cases
}
const NO_TERMINATOR: &[(&str, &str)] = &[
    (
        "emacs",
        "is handed one Emacs Lisp form as the value of `--eval`. A `--` in front of it would \
         become the form, and the package name is inside the form rather than beside it — \
         which is the same reason `argv_drift_tests` excuses emacs from the `--help` walk.",
    ),
    (
        "service",
        "uses init-provider commands as complete argv sequences. Their service name is an \
         operand of `rc-service`, `systemctl`, `launchctl`, or `sc`, not a package-manager \
         operand that the shared package terminator can precede.",
    ),
];
fn operands_outside_the_terminator(case: &ArgvCase, calls: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for call in calls {
        let tokens: Vec<&str> = call.split_whitespace().collect();
        let Some((program, args)) = tokens.split_first() else {
            continue;
        };
        if !shall::core::argv::terminates_options(program) {
            continue;
        }
        // A terminator anywhere means the options ended; everything after it is an operand
        // by definition and needs nothing more.
        if args.contains(&"--") {
            continue;
        }
        // An operand is a token that is not itself an option and that carries the subject
        // the declaration named — `nixpkgs#jq` is the operand for a `jq` declaration.
        let carries_subject = args
            .iter()
            .any(|a| !a.starts_with('-') && a.contains(case.subject));
        if !carries_subject {
            continue;
        }
        if NO_TERMINATOR.iter().any(|(b, _)| *b == case.backend) {
            continue;
        }
        out.push(format!(
            "{}: `{}` — `{}` ends its options at `--`",
            case.backend, call, program
        ));
    }
    out
}
fn check(
    backend: &str,
    verb: &str,
    expect: &Expect,
    outcome: shall::core::Result<()>,
    calls: &[String],
) {
    match expect {
        Expect::Runs(want) => {
            assert!(
                calls.iter().any(|c| c.contains(want)),
                "{backend}: {verb} ran no call containing `{want}`\n  calls: {calls:?}"
            );
        }
        Expect::RunsOneOf(wants) => {
            assert!(
                wants
                    .iter()
                    .any(|want| calls.iter().any(|c| c.contains(want))),
                "{backend}: {verb} ran no call containing one of {wants:?}\n  calls: {calls:?}"
            );
        }
        Expect::Unsupported => {
            assert!(
                matches!(outcome, Err(shall::core::Error::Unsupported(_))),
                "{backend}: this manager has no {verb} verb, so it must refuse with \
                 Unsupported — it returned {:?}",
                outcome.map(|()| "Ok")
            );
            assert!(
                calls.is_empty(),
                "{backend}: {verb} is unsupported and yet it ran something: {calls:?}"
            );
        }
        Expect::NoCommand(why) => {
            assert!(
                calls.is_empty(),
                "{backend}: {verb} is documented as running no command — \"{why}\" — and it \
                 ran {calls:?}. Either the backend grew a subprocess, in which case give it \
                 a `Runs` row, or the reason is now wrong."
            );
            assert!(
                why.len() > 40,
                "{backend}: {verb}'s no-command exemption has no reason worth the name"
            );
        }
    }
}
fn registry_source() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/backends/registry");
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .expect("the registry directory is readable")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "the registry directory holds no source");
    files
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(
            "
",
        )
        .replace("pub(super) fn ", "fn ")
        .replace("pub fn register_", "fn register_")
}
mod a_definition_that_does_not_say_so_leaves_the_built_in_alone;
mod a_definition_that_says_so_replaces_a_built_in;
mod a_dependency_probe_sends_the_operand_and_keeps_the_format_string;
mod a_generic_backend_carries_its_managers_exit_policy;
mod a_helm_plugin_declared_without_its_url_is_refused_by_name;
mod a_mix_archive_is_pinnable_and_its_removal_does_not_wait_for_an_answer;
mod a_repository_verb_runs_the_program_that_verb_needs;
mod a_required_version_fallback_is_an_operand_not_a_flag;
mod a_trailing_positional_version_lands_after_the_name;
mod a_version_after_the_name_keeps_the_terminator_unless_it_is_an_option;
mod cache_cleaning_runs_a_command_or_refuses_by_name;
mod every_backend_sends_the_argv_its_manager_expects;
mod every_registrar_gives_its_core_the_managers_exit_policy;
mod every_repo_row_runs_the_program_that_edits_that_managers_sources;
mod every_walk_of_the_registry_is_in_the_same_order;
mod no_backend_has_two_argv_rows;
mod no_self_resolving_system_manager_re_derives_a_dependency_closure;
mod only_the_backends_that_cannot_know_your_intent_opt_out_of_adopt;
mod registry_capability_matrix;
mod the_keyword_backends_can_all_do_the_thing_their_keyword_names;
mod the_terminator_check_can_actually_fail;
mod winget_adoption_reads_what_it_can_reinstall_not_what_it_can_see;
mod winget_is_not_among_the_managers_that_adopt_their_whole_listing;
