//! Shared test doubles: snapshot providers, task provisioners, and the scaffolding eighteen
//! integration binaries build an `App` out of.
//!
//! **This file is `mock_providers/mod.rs` rather than `mock_providers.rs`, and the directory is
//! the point.** Cargo auto-discovers every `tests/*.rs` as its own test target, so at the top
//! level this became a 716 KB binary containing **zero tests**, linked with `lto = true` and
//! `codegen-units = 1`, and its 312 lines were compiled nineteen times: once as that binary and
//! once inside each `mod mock_providers;` that includes it. A directory is not a target, and
//! `mod mock_providers;` resolves here unchanged — so no caller moved.
#![allow(clippy::field_reassign_with_default, dead_code)]

/// A backend that answers `pins_version` and `supports_purge` both ways and records what the
/// engine sent it. Its own file because it is a *backend*, not one more provider double, and
/// because the engine's argv decisions are the one subject `MockExecutor` sits too low to reach.
pub mod recording_backend;

use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, Utc};
use dashmap::DashMap;
use shall::app::scheduler::{Provisioned, Reading, TaskProvisioner, TaskTarget};
use shall::app::App;
use shall::config::declaration::ScheduleConfig;
use shall::config::Config;
use shall::core::executor::MockExecutor;
use shall::core::{
    CommandExecutor, Error, PackageSpec, Result, Snapshot, SnapshotProvider, StateRegistry,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

#[allow(dead_code)]
pub struct TestKernel {
    pub app: App,
    pub mock_executor: Arc<MockExecutor>,
    pub tmp: TempDir,
    pub state: Arc<Mutex<StateRegistry>>,
    vfs: Arc<DashMap<std::path::PathBuf, String>>,
    lock_map: Arc<DashMap<String, Arc<tokio::sync::Mutex<()>>>>,
}

/// Shall no longer injects a commit identity (a signed commit must not be authored by a name
/// nobody owns), so a fixture that commits fails on any host without a global `user.email` —
/// which is every CI runner (S33). The absent config paths keep the developer's own
/// `~/.gitconfig` out too: a host that signs every commit would fail these at `git commit`.
///
/// Twinned in `src/core/git.rs`'s test module, which is a separate binary and cannot link to
/// this one. Change one, change the other.
fn hermetic_git_env() {
    static HERMETIC: std::sync::Once = std::sync::Once::new();
    HERMETIC.call_once(|| {
        for (k, v) in [
            ("GIT_AUTHOR_NAME", "shall-tests"),
            ("GIT_AUTHOR_EMAIL", "test@example.invalid"),
            ("GIT_COMMITTER_NAME", "shall-tests"),
            ("GIT_COMMITTER_EMAIL", "test@example.invalid"),
            ("GIT_CONFIG_GLOBAL", "shall-tests-absent-gitconfig"),
            ("GIT_CONFIG_SYSTEM", "shall-tests-absent-gitconfig"),
        ] {
            std::env::set_var(k, v);
        }
    });
}

impl TestKernel {
    pub async fn new() -> Self {
        // Before anything can commit: set in the kernel, not in the one test that noticed,
        // so a fixture added later cannot forget it.
        hermetic_git_env();

        let tmp = tempfile::Builder::new()
            .prefix("shall_hermetic_")
            .tempdir()
            .expect("Failed to create test sandbox directory.");

        let registry_path = tmp.path().join("registry.json");

        // S11: every path under the sandbox, in one call. Setting the roots by hand here is
        // what let the sibling fixture forget `data_root` and write to real user state.
        // Not a dry run: the executor below is a mock, so nothing reaches the machine, and a
        // fixture that claims preview-only while asserting mutations happened is a fixture
        // that cannot see a preview-only bug (S25). A test that wants the preview path clones
        // this config and sets `dry_run`.
        let config = Config::sandboxed(tmp.path());

        // The II.1 layout, because the resolver now reads a repo rather than a folder of
        // manifests. `priority` says which package managers this machine uses: without it
        // Shall refuses to guess, so a fixture without one is a fixture that cannot resolve
        // anything. These are the three the mock executor pretends to have.
        std::fs::write(tmp.path().join("priority"), "apt\nbrew\ncargo\n")
            .expect("Failed to write test `priority`.");
        std::fs::create_dir_all(tmp.path().join("modules"))
            .expect("Failed to create test `modules/`.");
        std::fs::create_dir_all(tmp.path().join("profiles"))
            .expect("Failed to create test `profiles/`.");
        // A profile has to be active or there is nowhere for a line to go: a module no
        // profile reaches is one Shall never reads, so writing to one is refused rather
        // than done silently. A fixture with nothing active is a fixture that cannot
        // install. Tests that care about a specific profile overwrite these two files.
        std::fs::write(tmp.path().join("profiles/Main"), "")
            .expect("Failed to write test profile.");
        std::fs::write(tmp.path().join("active"), "Main\n")
            .expect("Failed to write test `active`.");

        let vfs = Arc::new(DashMap::new());
        let lock_map = Arc::new(DashMap::new());

        let mock_executor = Arc::new(MockExecutor::new(vfs.clone()));
        // **Every manager the suite probes, seeded once.**
        //
        // `check_command` answers `true` for an unseeded name and records the probe, so a fixture
        // that seeded three left ~59 names falling through on every test that asked. Those probes
        // are not defects — `true` is deliberately what an unopinionated test means — but leaving
        // them unanswered made the fall-through ledger unreadable: 134 of the 352 tests that
        // reported at least one unanswered command reported *only* probes, which is noise in the
        // exact measurement that says which tests are asserting against an empty string they never
        // wrote (`#40`).
        //
        // Seeding here states the assumption the tests were already making. It changes what the
        // ledger holds, not what any verdict is: `check_command`'s default was `true` and this is
        // still `true`. Re-measured after `V.239` — see that entry for the counts.
        mock_executor.set_command_exists("apk", true);
        mock_executor.set_command_exists("apt", true);
        mock_executor.set_command_exists("asdf", true);
        mock_executor.set_command_exists("brew", true);
        mock_executor.set_command_exists("btrfs", true);
        mock_executor.set_command_exists("bun", true);
        mock_executor.set_command_exists("cabal", true);
        mock_executor.set_command_exists("cargo", true);
        mock_executor.set_command_exists("code", true);
        mock_executor.set_command_exists("composer", true);
        mock_executor.set_command_exists("conda", true);
        mock_executor.set_command_exists("dart", true);
        mock_executor.set_command_exists("dkms", true);
        mock_executor.set_command_exists("dnf", true);
        mock_executor.set_command_exists("dotnet", true);
        mock_executor.set_command_exists("dpkg", true);
        mock_executor.set_command_exists("emacs", true);
        mock_executor.set_command_exists("emerge", true);
        mock_executor.set_command_exists("eopkg", true);
        mock_executor.set_command_exists("flatpak", true);
        mock_executor.set_command_exists("gem", true);
        mock_executor.set_command_exists("go", true);
        mock_executor.set_command_exists("gsettings", true);
        mock_executor.set_command_exists("guix", true);
        mock_executor.set_command_exists("helm", true);
        mock_executor.set_command_exists("kubectl", true);
        mock_executor.set_command_exists("kubectl-krew", true);
        mock_executor.set_command_exists("luarocks", true);
        mock_executor.set_command_exists("lvs", true);
        mock_executor.set_command_exists("mise", true);
        mock_executor.set_command_exists("mix", true);
        mock_executor.set_command_exists("moss", true);
        mock_executor.set_command_exists("nimble", true);
        mock_executor.set_command_exists("nix", true);
        mock_executor.set_command_exists("npm", true);
        mock_executor.set_command_exists("opam", true);
        mock_executor.set_command_exists("pacman", true);
        mock_executor.set_command_exists("paru", true);
        mock_executor.set_command_exists("pip", true);
        mock_executor.set_command_exists("pipx", true);
        mock_executor.set_command_exists("pixi", true);
        mock_executor.set_command_exists("pnpm", true);
        mock_executor.set_command_exists("probe-detect", true);
        mock_executor.set_command_exists("pwsh", true);
        mock_executor.set_command_exists("rpm", true);
        mock_executor.set_command_exists("rustup", true);
        mock_executor.set_command_exists("slackpkg", true);
        mock_executor.set_command_exists("snap", true);
        mock_executor.set_command_exists("spack", true);
        mock_executor.set_command_exists("stack", true);
        mock_executor.set_command_exists("systemctl", true);
        mock_executor.set_command_exists("testinit-detect", true);
        mock_executor.set_command_exists("timeshift", true);
        mock_executor.set_command_exists("uv", true);
        mock_executor.set_command_exists("xbps-install", true);
        mock_executor.set_command_exists("yarn", true);
        mock_executor.set_command_exists("yay", true);
        mock_executor.set_command_exists("zfs", true);
        mock_executor.set_command_exists("zypper", true);

        // **What this machine holds, stated once instead of fallen through.**
        //
        // `priority` here is `apt`, `brew`, `cargo`, and every reconcile asks those managers what
        // they hold; the snapshot provider is asked what snapshots exist; a bare `absent:` name is
        // hunted across the rest. None of that is a claim about the product's argv, it is the
        // machine, so it is registered as a standing fact rather than per test — see
        // `MockExecutor::set_machine_response`.
        //
        // **A test that needs this machine to hold something overrides the key it cares about**
        // (`brew_holds` for brew), and its own registration is a normal one, held to the unused
        // check like every other. The per-package answers stay per test: `brew info -- pkg`,
        // `zypper search`, every install and every removal are the test's subject, not the
        // fixture's.
        let snapshot_listing = snapshot_listing_for_this_host();
        for listing in [
            "brew list --versions",
            "cargo install --list",
            "dpkg-query -W -f=${db:Status-Status} ${Package} ${Version}\\n",
            "pacman -Qmq",
            "snap list",
            // The snapshot **listing** is the same platform question as the create argv below:
            // `timeshift --list` is the Linux row, and a `$HOME` that is not Linux gets
            // `tmutil listlocalsnapshots` or a `Get-ComputerRestorePoint` query instead. Seeding
            // only the Linux one is the `#175` defect in its other half.
            snapshot_listing.as_str(),
            "zypper search --installed-only",
        ] {
            mock_executor.set_machine_response(
                listing,
                Ok(shall::core::executor::DryRunOutput::new().into()),
            );
        }

        let executor = CommandExecutor::with_layer(
            true,
            false,
            mock_executor.clone(),
            vfs.clone(),
            lock_map.clone(),
        );

        let app = App::new_with_executor_and_state_path(config, executor, Some(registry_path))
            .await
            .expect("Failed to bootstrap Test Kernel.");

        let state = app.state.clone();

        Self {
            app,
            mock_executor,
            tmp,
            state,
            vfs,
            lock_map,
        }
    }

    /// A **second run** over this same sandbox: a fresh `App`, a fresh executor, the same
    /// files, the same state registry, and the same mock machine underneath.
    ///
    /// The executor is rebuilt rather than duplicated because `CommandExecutor::duplicate` is
    /// `clone`, and the installed-listing memo is an `Arc` inside it — so a second sync taken
    /// from a duplicate answers "is it installed?" from the first sync's answer, which is the
    /// one question a convergence test exists to ask twice. The `vfs` and `lock_map` are
    /// shared, because those are the machine.
    #[allow(dead_code)]
    pub async fn second_run(&self) -> App {
        let executor = CommandExecutor::with_layer(
            true,
            false,
            self.mock_executor.clone(),
            self.vfs.clone(),
            self.lock_map.clone(),
        );
        App::new_with_executor_and_state_path(
            (*self.app.config).clone(),
            executor,
            Some(self.tmp.path().join("registry.json")),
        )
        .await
        .expect("a second run bootstraps over the same files")
    }

    /// The same sandbox, previewed: an `App` over these exact files with `--dry-run` set.
    ///
    /// A second `App` rather than a mutated one, because `config` is shared by `Arc` and
    /// flipping the flag in place would flip it for everything already holding a clone.
    #[allow(dead_code)]
    pub async fn previewing(&self) -> App {
        let mut config = (*self.app.config).clone();
        config.dry_run = true;
        App::new_with_executor_and_state_path(
            config,
            self.app.executor.clone(),
            Some(self.tmp.path().join("registry.json")),
        )
        .await
        .expect("Failed to bootstrap the previewing Test Kernel.")
    }

    #[allow(dead_code)]
    pub async fn assert_called(&self, command_fragment: &str) {
        let calls: Vec<String> = self.mock_executor.get_calls().await;
        assert!(
            calls.iter().any(|c| c.contains(command_fragment)),
            "\nCOMMAND VERIFICATION FAILED\nExpected Fragment: '{}'\nActual Call Log:   {:?}\n",
            command_fragment,
            calls
        );
    }

    /// **Say that this machine's manager has `name`, on the manager this machine can use.**
    ///
    /// The bare-name resolver asks every manager in `priority` in turn whether it has the name,
    /// and the first one that answers wins — so a fixture that only teaches `apt-cache search`
    /// is a fixture that works on Linux and nowhere else (#170). That is not a stub that fails
    /// loudly; it is one that silently leaves the chain, because `apt` is **not registered as a
    /// backend off Linux at all** (`registry/mod.rs`: the native managers are behind
    /// `cfg!(target_os = "linux")`), so the line has nothing to resolve against and the test
    /// fails with `no package manager this line accepts has …` — a sentence about a package that
    /// is perfectly findable, on a machine the fixture lied about.
    ///
    /// `apt` on Linux, `brew` everywhere else: `brew` is registered unconditionally and is the
    /// manager that really exists on the hosts the CI legs use.
    pub async fn this_machines_manager_has(&self, name: &str) {
        // The probe is built through `push_names`, the way the backend builds it, not spelled by
        // hand. The hand-spelled version produced `brew search dos2unix`, but brew terminates
        // options, so the product sends `brew search -- dos2unix` and the stub went unmatched on
        // macOS exactly as if it had not been written (#170; the `--` is asserted by
        // `tests/backends/brew/every_brew_command_ends_its_options_before_the_name.rs`).
        let (probe, line) = if cfg!(target_os = "linux") {
            let mut args = vec!["search".to_string()];
            shall::core::argv::push_names(&mut args, "apt-cache", [name]);
            (
                format!("apt-cache {}", args.join(" ")),
                format!("{name} - a program this machine has\n"),
            )
        } else {
            let mut args = vec!["search".to_string()];
            shall::core::argv::push_names(&mut args, "brew", [name]);
            (
                format!("brew {}", args.join(" ")),
                format!("==> Formulae\n{name}\n"),
            )
        };
        self.mock_executor.set_response(
            &probe,
            Ok(shall::core::executor::DryRunOutput {
                stdout: line.into_bytes(),
                stderr: Vec::new(),
            }
            .into()),
        );
    }

    /// Register the snapshot **listing** this host's provider asks for, the same way and for the
    /// same reason as [`stub_the_pre_sync_snapshot`](Self::stub_the_pre_sync_snapshot) (#175).
    ///
    /// These two are one defect in two halves and both were Linux literals: the create argv and
    /// the listing. Fixing only the create left `sudo -n timeshift --list` registered on a Mac,
    /// which is the same unused-stub failure with a different string — measured on the first run
    /// after the create half landed, in eight tests.
    pub async fn stub_the_snapshot_listing(&self) {
        let argv = snapshot_listing_for_this_host();
        if argv.is_empty() {
            return;
        }
        self.mock_executor
            .set_machine_response(&argv, Ok(shall::core::executor::DryRunOutput::new().into()));
    }

    /// **Register the pre-sync snapshot this host really takes (#175).**
    ///
    /// A `sync` with work to do takes a safety snapshot first, and *which program takes it* is
    /// the product's decision, not the test's: `timeshift` on Linux, `tmutil` on macOS and
    /// `powershell … Checkpoint-Computer` on Windows, chosen from the built-in rows by the same
    /// `os =` filter and the same `detect` probe every other adapter uses
    /// (`src/core/snapshot_builtins.toml`).
    ///
    /// So a test that writes one of those three strings as a literal is a test that works on the
    /// platform its author was sitting at. Everywhere else the product asks a program the stub
    /// does not name, the call falls through to the empty-success default, and the unused-stub
    /// check — correctly — fails the test. That is eight macOS failures and most of the Windows
    /// ones saying the same sentence about `sudo -n timeshift --create`, and the message blames
    /// the assertion rather than the fixture.
    ///
    /// **The answer is asked of the product, not copied from it.** A second table of three
    /// program names here would be free to drift from the one in `snapshot_builtins.toml` the day
    /// somebody changes a flag, and it would drift silently — which is the defect being fixed.
    /// `SnapshotManager::choose` is the function `sync` itself calls, over the same rows and the
    /// same mock machine, so this registers whatever that run is about to ask for.
    pub async fn stub_the_pre_sync_snapshot(&self) {
        use shall::core::snapshot::{ConfigSnapshotProvider, SnapshotManager};

        let providers: Vec<Box<dyn shall::core::SnapshotProvider>> =
            shall::core::snapshot::builtin_defs_for_a_test(&self.app.config)
                .into_iter()
                .map(|def| {
                    Box::new(ConfigSnapshotProvider {
                        executor: self.app.executor.clone(),
                        def,
                    }) as Box<dyn shall::core::SnapshotProvider>
                })
                .collect();
        let Some(chosen) =
            SnapshotManager::choose(providers, &self.app.config.snapshot_priority).await
        else {
            // No provider on this machine. That is a legitimate fixture — `tests/app/sync/mod.rs`
            // builds one on purpose, to test the "could not revert" half — and a test that takes
            // no snapshot has nothing to stub, so this cannot be an error.
            return;
        };
        // `choose` hands back the provider behind the trait object, and the argv seam is on the
        // row-driven implementation — which is every built-in, and the only kind `choose` returns
        // here because that is all this fixture built.
        let config_provider = chosen
            .as_any()
            .downcast_ref::<ConfigSnapshotProvider>()
            .expect("a built-in provider is a ConfigSnapshotProvider");
        for argv in config_provider.create_argv_for_a_test() {
            self.mock_executor
                .set_machine_response(&argv, Ok(shall::core::executor::DryRunOutput::new().into()));
        }
    }
}

/// The listing argv of the snapshot provider this host would use, or the Linux row's when this
/// host has none.
///
/// **Not a second copy of the provider table**: it is read out of `snapshot_builtins.toml` by
/// the same loader the product uses, filtered by the same `os =` rule. A machine that can
/// snapshot gets its own program seeded; a machine that cannot gets a row that will never be
/// asked, which is exactly what `set_machine_response` is for — it is excused from the unused
/// check because it is a fact about the machine rather than a claim about the product's argv.
pub fn snapshot_listing_for_this_host() -> String {
    let defs = shall::core::snapshot::builtin_defs_for_a_test(&shall::config::Config::default());
    let Some(def) = defs
        .iter()
        .find(|d| !d.detect.is_empty() && d.os.as_deref() == Some(std::env::consts::OS))
        .or_else(|| defs.iter().find(|d| d.name == "timeshift"))
    else {
        return String::new();
    };
    // No `{source}` substitution on purpose: only the btrfs row carries a placeholder in `list`,
    // and its argv is a fixed `subvolume list /`. A function that pretended to substitute one
    // would invite a row whose placeholder it does not know about.
    let Some((prog, rest)) = def.list.split_first() else {
        return String::new();
    };
    let refs: Vec<&str> = rest.iter().map(String::as_str).collect();
    shall::core::executor::CommandExecutor::as_launched(prog, &refs, true)
}

#[allow(dead_code)]
pub struct MockSnapshotProvider {
    pub store: Arc<Mutex<Vec<Snapshot>>>,
    pub deletions: Arc<Mutex<Vec<String>>>,
}

#[allow(dead_code)]
impl Default for MockSnapshotProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSnapshotProvider {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(Vec::new())),
            deletions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn add_historical_snapshot(&self, id: &str, days_ago: i64) {
        let timestamp = Utc::now() - ChronoDuration::days(days_ago);
        let mut list = self.store.lock().await;
        list.push(Snapshot {
            id: id.to_string(),
            timestamp: timestamp.to_rfc3339(),
            description: format!("Historical state from {} days ago", days_ago),
            backend: "mock".into(),
        });
    }
}

#[async_trait]
impl SnapshotProvider for MockSnapshotProvider {
    fn name(&self) -> &str {
        "mock"
    }
    async fn is_available(&self) -> bool {
        true
    }
    async fn create(&self, label: shall::core::snapshot::SnapshotLabel) -> Result<Snapshot> {
        let s = Snapshot {
            id: format!("snap_{}", Utc::now().timestamp()),
            timestamp: Utc::now().to_rfc3339(),
            description: label.to_string(),
            backend: "mock".into(),
        };
        self.store.lock().await.push(s.clone());
        Ok(s)
    }
    async fn list(&self) -> Result<Vec<Snapshot>> {
        Ok(self.store.lock().await.clone())
    }
    async fn delete(&self, id: &str) -> Result<()> {
        let mut list = self.store.lock().await;
        if let Some(pos) = list.iter().position(|s| s.id == id) {
            list.remove(pos);
            self.deletions.lock().await.push(id.to_string());
            Ok(())
        } else {
            Err(Error::Snapshot(format!(
                "Mock Logic Failure: Snapshot {} missing.",
                id
            )))
        }
    }
    fn restore_capability(&self) -> shall::core::snapshot::RestoreCapability {
        shall::core::snapshot::RestoreCapability::Live
    }
    async fn restore(&self, _id: &str) -> Result<()> {
        Ok(())
    }
}

#[allow(dead_code)]
pub struct MockTaskProvisioner {
    pub active_tasks: Arc<Mutex<HashMap<String, ScheduleConfig>>>,
}

#[allow(dead_code)]
impl Default for MockTaskProvisioner {
    fn default() -> Self {
        Self::new()
    }
}

impl MockTaskProvisioner {
    pub fn new() -> Self {
        Self {
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

fn account_marker(target: &TaskTarget) -> String {
    match &target.account {
        None => String::new(),
        Some(account) => format!(" @user={}", account.name()),
    }
}

pub fn mock_task_key(name: &str, target: &TaskTarget) -> String {
    match &target.account {
        None => name.to_string(),
        Some(account) => format!("{name}@user={}", account.name()),
    }
}

#[async_trait]
impl TaskProvisioner for MockTaskProvisioner {
    async fn add_task(
        &self,
        _executor: &CommandExecutor,
        config: &ScheduleConfig,
        _shall_path: &Path,
        target: &TaskTarget,
    ) -> Result<()> {
        let mut map = self.active_tasks.lock().await;
        map.insert(mock_task_key(&config.name, target), config.clone());
        Ok(())
    }
    async fn remove_task(
        &self,
        _executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> Result<()> {
        let mut map = self.active_tasks.lock().await;
        map.remove(&mock_task_key(name, target));
        Ok(())
    }
    async fn is_task_active(
        &self,
        _executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> bool {
        let map = self.active_tasks.lock().await;
        map.contains_key(&mock_task_key(name, target))
    }
    /// The mock scheduler expresses everything, so nothing is refused here. A refusal belongs
    /// to the OS that cannot hold the option, and inventing one for the mock would make a test
    /// about the mock's imagination.
    fn refuse_unsupported(&self, _config: &ScheduleConfig) -> std::result::Result<(), String> {
        Ok(())
    }
    /// It stores the whole declaration, so what it would provision and what it holds are the
    /// same value — which is what makes it a mock and not a second implementation.
    fn rendered(
        &self,
        config: &ScheduleConfig,
        _shall_bin: &Path,
        target: &TaskTarget,
    ) -> std::result::Result<Provisioned, String> {
        Ok(Provisioned {
            spec: format!("{:?}{}", config, account_marker(target)),
            armed: config.enabled.unwrap_or(true),
        })
    }
    async fn read_task(
        &self,
        _executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> Reading {
        let map = self.active_tasks.lock().await;
        match map.get(&mock_task_key(name, target)) {
            Some(config) => Reading::Holds(Provisioned {
                spec: format!("{:?}{}", config, account_marker(target)),
                armed: config.enabled.unwrap_or(true),
            }),
            None => Reading::Absent,
        }
    }

    /// The reverse question, and the mock has to answer it rather than declining: a provisioner
    /// that only ever hears about declarations is what let an orphaned unit outlive its line
    /// unnoticed (#158). The names it holds are the keys it has registered for this target.
    async fn installed(
        &self,
        _executor: &CommandExecutor,
        target: &TaskTarget,
    ) -> Option<Vec<String>> {
        let map = self.active_tasks.lock().await;
        let suffix = match &target.account {
            None => String::new(),
            Some(account) => format!("@user={}", account.name()),
        };
        let mut names: Vec<String> = map
            .keys()
            .filter_map(|key| {
                let name = match suffix.is_empty() {
                    true => key.as_str(),
                    false => key.strip_suffix(&suffix)?,
                };
                // A key belonging to another account's target is not this one's.
                if name.contains("@user=") {
                    return None;
                }
                Some(name.to_string())
            })
            .collect();
        names.sort();
        Some(names)
    }
}

#[allow(dead_code)]
pub fn create_dummy_spec(name: &str, backend: &str, source: Option<&str>) -> PackageSpec {
    let mut options = shall::config::grammar::Options::default();
    if let Some(src) = source {
        options.set("__source", src);
    }
    PackageSpec {
        name: name.to_string(),
        backend: backend.to_string(),
        options,
        requires: Vec::new(),
        present: true,
    }
}

#[cfg(test)]
mod the_pre_sync_stub_is_this_hosts_program {
    /// **What #175 is about, asserted rather than assumed.** If this helper ever disagreed with
    /// the string the product sends, the fixture fix would put the same class of blindness back
    /// one level down — a helper that is right on the author's machine and wrong on the next.
    ///
    /// The expected value is written per-platform *here*, in the test, which is the one place it
    /// is legitimate: a test that names the platform it is running on is honest, a product
    /// fixture that silently assumes one is not.
    #[tokio::test]
    async fn the_stub_names_the_argv_this_platform_really_sends() {
        let kernel = crate::mock_providers::TestKernel::new().await;
        kernel.stub_the_pre_sync_snapshot().await;
        let registered = kernel.mock_executor.registered_registrations();

        // **The escalation prefix belongs in the expectation, not in a `cfg` the product does
        // not have.** A snapshot create is elevated on every platform, and `as_launched` puts
        // `sudo -n` on it wherever `escalates()` says so — which excludes Windows and root. This
        // arm is what caught that when it was first written; a test that left the prefix off was
        // asserting a string the product never sends.
        let escalated = |line: &str| {
            if cfg!(windows) || shall::core::CommandExecutor::is_root() {
                line.to_string()
            } else {
                format!("sudo -n {line}")
            }
        };
        let expected = if cfg!(target_os = "macos") {
            escalated("tmutil localsnapshot")
        } else if cfg!(target_os = "windows") {
            // Windows has no `sudo`, so this is already the whole line.
            escalated(
                "powershell -NoProfile -NonInteractive -Command Checkpoint-Computer -Description \
                 'Shall: pre_sync' -RestorePointType 'APPLICATION_INSTALL'; (Get-ComputerRestorePoint \
                 | Sort-Object SequenceNumber | Select-Object -Last 1 -ExpandProperty SequenceNumber)",
            )
        } else {
            escalated("timeshift --create --comments pre_sync --tags D")
        };
        assert!(
            registered.contains(&expected),
            "the fixture registered {registered:?}, and this platform's sync asks for `{expected}`"
        );
    }

    /// **The arm a Linux CI runner cannot otherwise see.**
    ///
    /// `this_machines_manager_has` picks `apt-cache search` on Linux and `brew search --`
    /// elsewhere, and the "elsewhere" arm is the one #170 is about — so it must be checkable
    /// from the Linux runner that exists, which this test is.
    ///
    /// **It is deliberately a comparison against the product's own argv, split across two files**
    /// so nobody rewrites both sides to agree: this test holds that the helper writes exactly the
    /// string this host resolves through, and
    /// `backends::brew::every_brew_command_ends_its_options_before_the_name` holds that brew's
    /// `Searchable::search` sends `brew search -- <name>`. Together they compose into "the stub
    /// this machine's manager really answers". The `--` is the byte the first version of this fix
    /// got wrong; the brew test is what would have caught it.
    #[tokio::test]
    async fn each_managers_bare_name_probe_is_the_one_the_helper_writes() {
        let kernel = crate::mock_providers::TestKernel::new().await;
        // The helper's whole job is to *write* a probe; this test reads the table rather than
        // provoking a call, so the unused-stub check is asked to stand down here and only here.
        kernel.mock_executor.allow_unmatched_registrations();
        kernel.this_machines_manager_has("dos2unix").await;
        let registered = kernel.mock_executor.registered_registrations();
        let expected = if cfg!(target_os = "linux") {
            "apt-cache search -- dos2unix"
        } else {
            "brew search -- dos2unix"
        };
        assert!(
            registered.iter().any(|r| r == expected),
            "the helper wrote {registered:?}, and this host resolves a bare name through `{expected}`"
        );
    }

    /// **All three arms, from one machine.** The defect #175 fixes is only *visible* on a host
    /// that is not the author's, so a test that can only run on the host it is testing is the
    /// shape that let it sit there. `applies_to(os)` takes the OS as a parameter for exactly this
    /// reason, and it is what makes the macOS and Windows arms checkable on CI's Linux runners.
    ///
    /// What this pins is narrower than the test above and deliberately so: not "the argv is
    /// right", but **"the row this host would get is the row for this host"** — the selection
    /// rule, which is the half that was platform-blind.
    #[test]
    fn each_platform_selects_its_own_snapshot_row() {
        use shall::core::adapter::AdapterRow;

        let defs =
            shall::core::snapshot::builtin_defs_for_a_test(&shall::config::Config::default());
        let program_for = |os: &str| {
            let mut names: Vec<&str> = defs
                .iter()
                .filter(|d| d.applies_to(os))
                .map(|d| d.detect.as_str())
                .collect();
            names.sort();
            names
        };

        // Two, not three: `builtin_defs_for_a_test` drops the zfs row when no dataset is
        // configured, because a provider that cannot name what to snapshot is a create that
        // fails rather than a safety net. That is the loader's job and this asserts it holds.
        assert_eq!(program_for("linux"), ["btrfs", "timeshift"]);
        assert_eq!(program_for("macos"), ["tmutil"]);
        assert_eq!(program_for("windows"), ["powershell"]);
        // And the probe the fixture writes is on the axis the product chooses on: a row is only
        // seeded for the host the suite runs on, which is what `snapshot_listing_for_this_host`
        // does and what the arms above say it will pick.
        assert!(
            program_for(std::env::consts::OS).contains(&"timeshift")
                || program_for(std::env::consts::OS).contains(&"tmutil")
                || program_for(std::env::consts::OS).contains(&"powershell"),
            "this host picks no snapshot program at all, so the fixture would seed none"
        );
    }
}
