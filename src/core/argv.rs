//! Where a package name stops being an option (II.12b).
//!
//! The grammar refuses a name that starts with `-`. That is the layer that can name the file
//! and the line, and it holds for every manager. This is the other layer: the set of flags
//! belongs to the manager and changes without us, so an invocation ends its own options before
//! the names begin — `apt-get install -y -- ripgrep`.
//!
//! Not every CLI has a `--`. A manager that does not honour it is listed here by name, and
//! **it is listed rather than assumed**: a `--` a manager parses as a package name turns every
//! install into a failure, which is worse than the leading-dash refusal the grammar already
//! made. The default is therefore "does not terminate", and a manager joins the terminating
//! set when someone has checked its argument parser.
//!
//! **One table, and every row carries its evidence.** There were two — `TERMINATES` and a
//! `#[cfg(test)]` `DOES_NOT_TERMINATE`, with a test whose whole job was to catch them
//! contradicting each other, and with half the production facts compiled only into tests. Two
//! tables that can disagree is the shape this repo keeps paying for. The rows are now one list
//! of `(binary, terminates, evidence)`, disagreement is unrepresentable, and the evidence
//! field is the rule at the top of this file made mechanical: a row says either what the tool
//! *said* when someone ran it, or that nobody has asked. The second kind is counted, and the
//! count may fall but never rise.

/// Why a row says what it says.
///
/// An exemption is a claim (E29), so the two kinds are distinct in the type rather than in a
/// comment somebody may not write. `Measured` carries the tool's own words — not a citation,
/// not a date, the sentence it printed. `Unasked` is an honest confession, and
/// [`UNASKED_CEILING`] is what stops the confessions accumulating.
#[derive(Debug, Clone, Copy)]
pub enum Evidence {
    /// Run, and this is what it answered.
    Measured(&'static str),
    /// Never run. Why it is nonetheless listed this way.
    Unasked(&'static str),
    /// **Run on more than one host, and they did not agree.**
    ///
    /// The table is one boolean per binary and the world is not, so a tool that honours the
    /// terminator on one platform and eats it on another cannot be described by any single
    /// value the probe will accept everywhere. The row takes the **refusing** answer — the same
    /// conservative merge the probe already applies across a binary's verbs, where one verb that
    /// swallows makes the terminator unsafe for the whole binary — and the probe stops calling
    /// the other host's reading a disagreement.
    ///
    /// **Only ever `false`**, asserted by `a_divergent_row_takes_the_refusing_answer`. A
    /// divergent row that claimed to terminate would be Shall emitting a `--` that some host
    /// reads as a package name, which is the failure this whole table exists to prevent; the
    /// exemption is for the safe direction and for nothing else.
    Divergent(&'static str),
}

/// Whether a binary's CLI ends option parsing at `--`, and how that is known.
///
/// Keyed on the binary actually invoked, not the Shall backend name, because that is what
/// parses the arguments (apt's installs run `apt-get`, its queries run `dpkg-query`).
pub struct Terminator {
    pub binary: &'static str,
    pub terminates: bool,
    pub evidence: Evidence,
}

/// A parser whose `--` handling comes from the C library rather than from the tool.
const GETOPT: Evidence = Evidence::Unasked(
    "delegates option parsing to getopt(3)/argp, where `--` is the terminator by definition. \
     Inferred from the parser it links against, not asked of the tool itself.",
);

pub const TERMINATORS: &[Terminator] = &[
    // ---- Debian/Ubuntu.
    row(
        "apt",
        true,
        Evidence::Measured(
            "asked in the ubuntu integration image, CI run 31730161038: the probe drove its verbs with an \
             operand behind a `--` and every one of them read the operand as a package name.",
        ),
    ),
    row("apt-get", true, GETOPT),
    row(
        "apt-cache",
        true,
        Evidence::Measured(
            "asked in the ubuntu integration image, CI run 31730161038, alongside `apt` — same answer, and \
             asked separately because it is a different binary with its own parser.",
        ),
    ),
    row("apt-mark", true, GETOPT),
    row("dpkg", true, GETOPT),
    row("dpkg-query", true, GETOPT),
    row("aptitude", true, GETOPT),
    // ---- Red Hat.
    //
    // **`dnf` is not one program.** Fedora 41 replaced the Python/argparse `dnf` with `dnf5`,
    // a C++ reimplementation with its own argument parser, and kept the name: `/usr/bin/dnf`
    // IS dnf5 there. So a row keyed on the binary name cannot tell the two apart, and the one
    // it must be right for is the one that refuses.
    row(
        "dnf",
        false,
        Evidence::Measured(
            "`dnf search -- jq` on Fedora answers `Unknown argument \"--\" for command \
             \"search\"`, and the install fails with it — because that `dnf` is dnf5. Listed \
             false for BOTH implementations rather than detected: dnf4 does not *need* the \
             terminator (the grammar already refuses a leading-dash name, II.12b), while dnf5 \
             is broken by it, so the safe answer is the same answer for both. Measured in CI \
             run 31393556390, 2026-08-10, where it failed every dnf install on the image.",
        ),
    ),
    row(
        "dnf5",
        false,
        Evidence::Measured(
            "the same refusal, under its own name: `Unknown argument \"--\" for command \
             \"search\"`. It was listed as terminating on the getopt inference, which is the \
             error this table's own header warns about — dnf5 links no getopt.",
        ),
    ),
    // **The siblings, and they are the same program.** On a Fedora with dnf5, `/usr/bin/yum`
    // and `/usr/bin/microdnf` are both dnf5 under another name — `yum` has been a symlink to
    // dnf since Fedora 22, and dnf5 provides `microdnf`. So a fix that moved only the row
    // spelled `dnf` would have left the identical defect live under two more spellings, on the
    // same machines, reachable by any config that names them. Where they are NOT dnf5
    // (RHEL/CentOS, where `yum` is dnf4) the terminator is defence-in-depth the grammar
    // already provides, so false is right there too.
    row(
        "yum",
        false,
        Evidence::Measured(
            "the same parser as `dnf`, because on Fedora it IS `dnf` — a symlink since \
             Fedora 22, and dnf5 since Fedora 41. Where it is dnf4 instead, dropping the \
             terminator costs nothing the grammar's leading-dash refusal does not already \
             cover (II.12b).",
        ),
    ),
    row(
        "microdnf",
        false,
        Evidence::Measured(
            "provided by dnf5 on any image that has dnf5, so it is the binary that answered \
             `Unknown argument \"--\"`. Listed with its two siblings rather than left on the \
             getopt inference that was wrong for all three.",
        ),
    ),
    row("rpm", true, GETOPT),
    // ---- Arch.
    row(
        "pacman",
        true,
        Evidence::Measured(
            "asked in the arch integration image, CI run 31730161038. Its own parser rather than getopt(3), \
             which is exactly why the inference it replaced was worth confirming.",
        ),
    ),
    row(
        "yay",
        true,
        Evidence::Measured(
            "asked in the arch integration image, 2026-08-14, the first run in which the AUR \
             helpers were installed anywhere: every verb that resolved the operand read it as a \
             package name behind a `--`. Measured both as root and as the unprivileged harness \
             user, which is worth recording — `pacman` on the same image is measurable only as \
             root, because it refuses before naming the operand, and the helpers do not.",
        ),
    ),
    row(
        "paru",
        true,
        Evidence::Measured(
            "asked in the arch integration image, 2026-08-14, alongside `yay` — same answer, and \
             asked separately because it is a different binary with its own parser (Rust, where \
             `yay` is Go; neither links getopt(3), which is what GETOPT had claimed of both).",
        ),
    ),
    row("pamac", true, GETOPT),
    // ---- Alpine, Void, SUSE, Gentoo.
    row(
        "apk",
        true,
        Evidence::Measured(
            "asked in the alpine integration image, CI run 31730161038 — a BusyBox userland, where the \
             getopt(3) inference is least safe and now does not have to be made.",
        ),
    ),
    row("xbps-install", true, GETOPT),
    row("xbps-remove", true, GETOPT),
    row("xbps-query", true, GETOPT),
    row(
        "zypper",
        true,
        Evidence::Measured(
            "asked in the opensuse integration image, CI run 31730161038: the operand behind `--` reached \
             the resolver as a package name on every verb the probe could drive.",
        ),
    ),
    row("emerge", true, GETOPT),
    row("eix", true, GETOPT),
    row("equery", true, GETOPT),
    // ---- Functional / sandboxed.
    row("nix-env", true, GETOPT),
    row(
        "nix",
        true,
        Evidence::Measured(
            "`nix profile remove -- <bogus>` answers `warning: Package name '<bogus>' does not \
             match any packages in the profile` — the name, not the terminator (tools image, \
             2026-08-04)",
        ),
    ),
    row("guix", true, GETOPT),
    row(
        "flatpak",
        true,
        Evidence::Measured(
            "`flatpak install -y -- <bogus>` answers `error: No remote refs found for '<bogus>'` \
             (tools image, 2026-08-04)",
        ),
    ),
    row("snap", true, GETOPT),
    // ---- Language ecosystems.
    row(
        "pip",
        true,
        Evidence::Measured(
            "`pip uninstall -y -- <bogus>` answers `WARNING: Skipping <bogus> as it is not \
             installed.` (tools image, 2026-08-04)",
        ),
    ),
    row("pip3", true, GETOPT),
    row(
        "pipx",
        true,
        Evidence::Measured(
            "`pipx install -- <bogus>` gets as far as `installing <bogus>...` (tools image, \
             2026-08-04)",
        ),
    ),
    row(
        "cargo",
        true,
        Evidence::Measured(
            "asked in all five green integration images, CI run 31730161038. clap, not getopt(3), and the \
             five hosts agreed — so this is a measurement rather than a family resemblance.",
        ),
    ),
    row(
        "npm",
        true,
        Evidence::Measured(
            "`npm uninstall -g -- <bogus>` answers `up to date` and exits 0, which is what \
             removing an absent package looks like (tools image, 2026-08-04)",
        ),
    ),
    row("pnpm", true, GETOPT),
    row("yarn", true, GETOPT),
    row(
        "bun",
        true,
        Evidence::Measured(
            "`bun add -g -- <bogus>` answers `error: GET \
             https://registry.npmjs.org/<bogus> - 404` (tools image, 2026-08-04)",
        ),
    ),
    row(
        "go",
        true,
        Evidence::Measured(
            "`go install -- <bogus>` answers `Try 'go install <bogus>@latest'` (tools image, \
             2026-08-04)",
        ),
    ),
    row(
        "luarocks",
        true,
        Evidence::Measured(
            "`luarocks install --` answers `Error: missing argument 'rock'` over usage \
             `<rock> [<version>]`, and `luarocks install -- <rock> <version>` is identical to \
             the same line without the terminator (tools image, 2026-08-04)",
        ),
    ),
    row(
        "opam",
        true,
        Evidence::Measured(
            "`opam install -y -- <bogus>` answers `[ERROR] No package named <bogus> found.` \
             (tools image, 2026-08-04)",
        ),
    ),
    row(
        "mix",
        true,
        Evidence::Measured(
            "`mix archive.install hex --force -- <bogus>` is identical to the same line without \
             the terminator, both naming the operand (tools image, 2026-08-04)",
        ),
    ),
    row(
        "cabal",
        true,
        Evidence::Measured(
            "`cabal install -- <bogus>` answers `cabal: Unknown package \"<bogus>\".` (tools \
             image, 2026-08-04)",
        ),
    ),
    row(
        "stack",
        false,
        Evidence::Divergent(
            "three runs, two answers. `stack install -- <bogus>` is identical to the same line \
             without the terminator on the tools image (2026-08-04) and on ubuntu-latest (run \
             31517118980), and on windows-latest the same argv reported `swallows` — the \
             terminator changed what stack did with its operand (run 31458415385). Both readings \
             are real, so the row takes the refusing one: the cost of being wrong that way is a \
             `--` Shall does not send, and the cost of being wrong the other way is an install \
             that names no package at all",
        ),
    ),
    // ---- Conda-likes.
    row(
        "conda",
        true,
        Evidence::Unasked(
            "`conda install -y -- <bogus>` spent its output on channel and platform banners and \
             was cut off before any verdict on the operand (tools image, 2026-08-04).",
        ),
    ),
    row("mamba", true, GETOPT),
    row("micromamba", true, GETOPT),
    row(
        "uv",
        true,
        Evidence::Measured(
            "`uv tool install -- <bogus>` answers `Because <bogus> was not found in the package \
             registry` (tools image, 2026-08-04)",
        ),
    ),
    row(
        "pixi",
        true,
        Evidence::Measured(
            "`pixi global install -- <bogus>` answers `Couldn't install environment <bogus>` \
             (tools image, 2026-08-04)",
        ),
    ),
    row(
        "mise",
        true,
        Evidence::Measured(
            "`mise use -g -- <bogus>` answers `Failed to install <bogus>@latest: <bogus> not \
             found in mise tool registry` (tools image, 2026-08-04)",
        ),
    ),
    // ---- macOS / BSD / other distro managers.
    row("brew", true, GETOPT),
    row("port", true, GETOPT),
    row("pkgin", true, GETOPT),
    row("pkg", true, GETOPT),
    row("pkg_add", true, GETOPT),
    row("pkg_delete", true, GETOPT),
    row("eopkg", true, GETOPT),
    row(
        "slackpkg",
        false,
        Evidence::Measured(
            "`slackpkg search -- bc` answers `search: Ignoring extra arguments: bc`, then \
             `Looking for -- in package list ... No package name matches the pattern` — exit 0, \
             empty result, no error (slackware image, 2026-08-14). It is a shell script that \
             reads $1 as the pattern, so GETOPT was inferred from a parser it does not link \
             against. `install` survives the terminator only because it takes a list and drops \
             the operand that matches nothing; `search` takes one and the terminator IS that one.",
        ),
    ),
    // ---- Kubernetes, Dart, editors, init.
    row(
        "helm",
        true,
        Evidence::Measured(
            "`helm plugin uninstall -- <bogus>` answers `Error: Plugin: <bogus> not found` \
             (tools image, 2026-08-04)",
        ),
    ),
    row(
        "kubectl",
        true,
        Evidence::Measured(
            "the differential probe ran kubectl's real argv both ways in the tools image and the \
             two runs agreed in every signal — same exit code, same operand echo, no stray `--` \
             (tests/core/argv/terminator_probe_tests.rs, 2026-08-04)",
        ),
    ),
    row("krew", true, GETOPT),
    row(
        "dart",
        true,
        Evidence::Measured(
            "`dart pub global activate -- <pkg> <version>` is identical to the same line \
             without the terminator, and usage reads `activate <package> \
             [version-constraint]` (tools image, 2026-08-04)",
        ),
    ),
    row("flutter", true, GETOPT),
    row("emacs", true, GETOPT),
    row(
        "systemctl",
        true,
        Evidence::Measured(
            "the differential probe ran systemctl's real argv both ways in the tools image and \
             the two runs agreed in every signal (tests/core/argv/terminator_probe_tests.rs, 2026-08-04)",
        ),
    ),
    // ---- Go-flag parsers: `age` and `sops` are handed a source path a declaration chose.
    row("age", true, GETOPT),
    row("sops", true, GETOPT),
    // ---- PHP. **This row has been decided three times, twice on evidence that could not
    // decide it.** A bogus operand cannot answer the question at all: composer's answer to
    // `require <bogus>` is the same "could not find a matching version" whether it read the
    // terminator or dropped it, so both earlier readings — "byte-identical, therefore honoured"
    // and "byte-identical, therefore swallowed" — were the same non-measurement with opposite
    // signs. A **flag-shaped** operand is what separates them, because the two hypotheses
    // predict different tools doing different things.
    row(
        "composer",
        true,
        Evidence::Measured(
            "`composer global search --format=json -- --version` searches packagist for the \
             string and answers with `sebastian/version`; the same line without the terminator \
             answers `Composer version 2.10.2` and searches nothing. `global require` and \
             `global remove` flip the same way — with the terminator `--version` is a package \
             name they fail to resolve, without it they print the version banner and exit 0 \
             (composer 2.10.2, official image, 2026-08-14)",
        ),
    ),
    // ---- Measured NOT to terminate. Every one of these was in the terminating set at some
    // point, put there by family resemblance, and every one of them broke something.
    row(
        "asdf",
        false,
        Evidence::Measured(
            "`asdf install -- <bogus>`, `asdf uninstall -- <bogus>` and `asdf list -- <bogus>` \
             all answer `No such plugin: --`: it dispatches on $1 as the plugin name. Every \
             verb, not just install (tools image, 2026-07-28 and 2026-08-04)",
        ),
    ),
    row(
        "spack",
        false,
        Evidence::Measured(
            "`spack find -- <bogus>` answers `No package matches the query: -- <bogus>` and \
             `spack uninstall -y -- <bogus>` mangles it to `~~<bogus>`: the terminator is read \
             into the spec (tools image, 2026-07-28 and 2026-08-04)",
        ),
    ),
    row(
        "gem",
        false,
        Evidence::Measured(
            "RubyGems' `--` separates gem names from BUILD arguments, so it consumes the \
             operand on every verb: `gem install -- <bogus>` and `gem uninstall -- <bogus>` \
             answer `Please specify at least one gem name`, and `gem list -- <bogus>` silently \
             lists every gem instead of filtering — a wrong answer, not an error (tools image, \
             2026-08-04)",
        ),
    ),
    row(
        "nimble",
        false,
        Evidence::Measured(
            "nimble's `--` is RubyGems', not GNU's: on install it reaches the Nim compiler, \
             which answers `arguments can only be given if the '--run' option is selected` and \
             fails every build that produces a binary; on uninstall, list and search it answers \
             `Unknown option: --` outright (tools image, 2026-08-04)",
        ),
    ),
    // ---- Not asked here: Windows- and macOS-native tools, and the init wrappers. The
    // reasons are the parser's shape, which is why they were listed before anyone could run
    // them; the ratchet counts them until someone does.
    row(
        "winget",
        true,
        Evidence::Measured(
            "`winget install --silent --accept-source-agreements --accept-package-agreements -- \
             <bogus>` and the same line without the terminator are identical in every signal — \
             exit -1978335212 both ways, `No package found matching input criteria`, and no \
             mention of a bare `--` in either. Same for `winget uninstall --silent`. Listed \
             false on the shape of the parser until the probe ran on windows-latest and \
             disagreed (nightly run 31458415385, 2026-08-11)",
        ),
    ),
    row(
        "choco",
        true,
        Evidence::Measured(
            "`choco install -y -- <bogus>`, `choco uninstall -y -- <bogus>` and `choco search -r \
             -- <bogus>` are each identical to the same line without the terminator: same exit, \
             no stray `--` in the output, the operand echoed the same way. If choco read `--` as \
             a package id it would have had two names to fail on rather than one, and said so. \
             Listed false on the shape of the parser until the probe asked (nightly run \
             31458415385, 2026-08-11)",
        ),
    ),
    row(
        "scoop",
        false,
        Evidence::Unasked("a PowerShell script dispatching on $args[0]; `--` becomes an app name."),
    ),
    row(
        "mas",
        false,
        Evidence::Unasked("takes a numeric App Store id positionally, with no option terminator."),
    ),
    // Listed as NOT terminating on the reasoning that .NET takes the tool id positionally and
    // would read `--` as one. It does not: `System.CommandLine` ends option parsing there like
    // any GNU-style CLI, and the guess cost every `dotnet:` install the protection this file
    // exists to give. Found by `tests/core/argv/terminator_probe_tests.rs` on its first real run — an
    // unmeasured row, wrong, which is the entire reason the probe was written.
    row(
        "dotnet",
        true,
        Evidence::Measured(
            "`dotnet tool uninstall --global -- <bogus>` is identical to the same line without \
             the terminator, both answering `A tool with the package Id '<bogus>' could not be \
             found.` (.NET 8.0.129, tools image, 2026-08-04)",
        ),
    ),
    row(
        "pwsh",
        false,
        Evidence::Unasked("is handed a `-Command` script, so there is no operand to protect."),
    ),
    row(
        "powershell",
        false,
        Evidence::Unasked("is handed a `-Command` script, so there is no operand to protect."),
    ),
    row(
        "code",
        false,
        Evidence::Unasked(
            "takes an extension id as the *value* of `--install-extension`, never as a \
             positional, so a `--` in front of it would become the value.",
        ),
    ),
    row(
        "gsettings",
        false,
        Evidence::Unasked(
            "dispatches on argv[1] by hand — no getopt, so a bare `--` is read as the command \
             name and the call fails before it reaches the schema.",
        ),
    ),
    row(
        "sc",
        false,
        Evidence::Unasked(
            "dispatches on argv[1] by hand with no getopt; `sc query --help` tries to query a \
             service literally named `--help`.",
        ),
    ),
    row(
        "launchctl",
        true,
        Evidence::Measured(
            "all four verbs Shall drives are identical with and without the terminator on \
             macos-latest: `load -w -- <bogus>` and `unload -w -- <bogus>` both answer `Load \
             failed: 5: Input/output error` at exit 0, `start -- <bogus>` and `stop -- <bogus>` \
             both exit 3 in silence, and none of the four mentions a stray `--`. Listed false on \
             the shape of the parser — it does take the service positionally, and that turned \
             out not to settle the question (nightly run 31458415385, 2026-08-11)",
        ),
    ),
    // **The four init-script rows, and the one inference that decided all of them.**
    // All four were listed false on the same reasoning — *it is a shell script taking the
    // service positionally, so there is no place a terminator could go* — which is the reasoning
    // `launchctl` above them already records being refuted by measurement. Asked, the four split
    // two and two, so the inference was not merely wrong, it was uninformative: a positional
    // operand says nothing about whether the script's own argument walk skips a leading `--`.
    // Whichever way a row lands, it lands on the tool's own words now.
    row(
        "rc-service",
        true,
        Evidence::Measured(
            "asked in the gentoo integration image, nightly run 31925296671: `rc-service -- \
             <bogus> start` and `... stop` are byte-identical with and without the terminator, \
             both exiting 1 with `* rc-service: service '<bogus>' does not exist` — the operand \
             is read as the service name, not as a stray option.",
        ),
    ),
    row(
        "rc-update",
        true,
        Evidence::Measured(
            "the same run, both verbs Shall drives: `rc-update add -- <bogus> default` answers \
             `* rc-update: service '<bogus>' does not exist` and `rc-update del -- <bogus> \
             default` answers `* rc-update: service '<bogus>' is not in the runlevel 'default'`, \
             each identical to the same call without the `--`.",
        ),
    ),
    // **And the SysVinit pair answered the other way, which is why they were asked separately.**
    // The OpenRC rows above and these two carry the same one-line description — a shell script
    // taking the service positionally — and that description predicts the same answer for all
    // four. It is wrong for all four: it does not say whether the script's own argument walk
    // skips a leading `--`, and OpenRC's does while SysVinit's does not. Both halves are now
    // measured, so neither rests on the shape of the parser.
    row(
        "update-rc.d",
        false,
        Evidence::Measured(
            "asked on ubuntu 24.04: `update-rc.d -- <bogus> defaults` answers `update-rc.d: \
             error: unknown option` and prints its usage, where the same call without the \
             terminator gets as far as `unable to read /etc/init.d/<bogus>`. The `--` is rejected \
             as an option, not consumed as one.",
        ),
    ),
    row(
        "service",
        false,
        Evidence::Measured(
            "the same host: `service -- <bogus> start` answers `--: unrecognized service` where \
             the call without it answers `<bogus>: unrecognized service`. The terminator is read \
             as the service NAME — the worst of the three outcomes, because it neither errors on \
             the `--` nor protects the operand.",
        ),
    ),
];

/// A `const fn` so the table above reads as data rather than as struct literals.
const fn row(binary: &'static str, terminates: bool, evidence: Evidence) -> Terminator {
    Terminator {
        binary,
        terminates,
        evidence,
    }
}

/// Whether `binary` ends its option parsing at `--`.
pub fn terminates_options(binary: &str) -> bool {
    let base = base_name(binary);
    TERMINATORS
        .iter()
        .find(|t| t.binary == base)
        .is_some_and(|t| t.terminates)
}

/// The bare program name, so an absolute path (`/usr/bin/apt-get`, `C:\...\scoop.exe`) is
/// looked up as the program it is.
fn base_name(binary: &str) -> &str {
    let after_dir = binary.rsplit(['/', '\\']).next().unwrap_or(binary);
    after_dir.strip_suffix(".exe").unwrap_or(after_dir)
}

/// Every binary this table has an answer for, with that answer. For the probe, which exists to
/// ask the tools themselves whether the answers are still true.
pub fn known_terminator_claims() -> Vec<(&'static str, bool)> {
    TERMINATORS
        .iter()
        .map(|t| (t.binary, t.terminates))
        .collect()
}

/// Why this row says what it says, and whether anyone has run the tool to find out.
///
/// Read by the probe, so a disagreement between the table and the tool is reported next to the
/// reason the table believed itself — which is usually the whole diagnosis. A stored fact that
/// nothing reads is the shape this file exists to remove; storing evidence and never printing
/// it would have been the same mistake in a new place.
pub fn terminator_evidence(binary: &str) -> Option<(bool, &'static str)> {
    let base = base_name(binary);
    TERMINATORS
        .iter()
        .find(|t| t.binary == base)
        .map(|t| match t.evidence {
            Evidence::Measured(w) | Evidence::Divergent(w) => (true, w),
            Evidence::Unasked(w) => (false, w),
        })
}

/// Whether this row's answer is known to differ between hosts.
///
/// Read by `tests/core/argv/terminator_probe_tests.rs`, which asserts the table against the tools
/// themselves and otherwise has no way to be right on both platforms at once: whichever value
/// the row carries, one host's probe reports a disagreement. See [`Evidence::Divergent`].
pub fn terminator_answer_differs_by_host(binary: &str) -> bool {
    let base = base_name(binary);
    TERMINATORS
        .iter()
        .find(|t| t.binary == base)
        .is_some_and(|t| matches!(t.evidence, Evidence::Divergent(_)))
}

/// Append package names to an argument list, ending the manager's options first where the
/// manager honours `--`.
pub fn push_names<S: AsRef<str>>(
    args: &mut Vec<String>,
    binary: &str,
    names: impl IntoIterator<Item = S>,
) {
    let mut names = names.into_iter().peekable();
    if names.peek().is_none() {
        return;
    }
    if terminates_options(binary) {
        args.push("--".to_string());
    }
    for n in names {
        args.push(n.as_ref().to_string());
    }
}
