//! The value types a data-driven backend is described with: version pins, listings,
//! probe shapes and the exit-code/prefix tables the TOML file fills in.

use super::*;

/// How a backend expresses an exact version at install time, for reproducible
/// (locked) installs. `{name}` / `{version}` are substituted.
///
/// **The variants say where the version goes, and nothing else.** Whether it is an *option* —
/// the one property that decides if the `--` terminator can precede it — is read off the
/// tokens by [`emits_trailing_option`](VersionPin::emits_trailing_option), because an option
/// starts with `-` and a version does not. It was a third variant for a while, and the third
/// variant is what went wrong: `Flag`, `TrailingPositional` and `RequiredFlag` had
/// character-for-character the same body, so the argv they built was identical and only the
/// *label* decided the terminator. Three backends carrying a bare positional — `luarocks`,
/// `mix`, `pub` — were spread across two of those labels, and the two that guessed wrong
/// dropped the terminator on pinned installs and kept it on unpinned ones. A fact the data
/// already states cannot be restated by hand without eventually disagreeing with itself.
#[derive(Debug, Clone)]
pub enum VersionPin {
    /// One token: apt `name=version`, pip `name==version`, bun `name@version`.
    Inline(String),
    /// Args placed **before** the name, which therefore stays behind the `--` terminator:
    /// cargo `install --version 13.0.0 -- ripgrep`. Available whenever the flag belongs to the
    /// subcommand rather than to the operand, and it is the safer of the two placements.
    ///
    /// **A leading flag cannot be batched.** `cargo install --version 1.0 --version 2.0 -- a b`
    /// is not two pinned installs, it is nonsense — so a backend using this installs one spec
    /// per command. [`After`](VersionPin::After) and [`Inline`](VersionPin::Inline) carry their
    /// version beside or inside the operand and batch fine.
    Before(Vec<String>),
    /// Args placed **after** the name: gem `jq -v 1.6` (an option), luarocks `jq 1.6`,
    /// mix `phx_new 1.6.16`, pub `webdev 2.7.0` (all three operands).
    After {
        args: Vec<String>,
        /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
        /// test crate links the library without `cfg(test)`, so a private item is invisible there.
        /// What to ask for when the declaration pins no version, for a manager that
        /// **refuses to install without one**.
        ///
        /// `None` for every ordinary manager: "no version" means "whatever is current", which
        /// is what an unadorned install already does. asdf is the one that needs it —
        /// `asdf install nodejs` answers `No versions specified for nodejs in config files or
        /// environment` and `asdf install nodejs latest` installs (measured, `tools` image
        /// 2026-07-29) — and `luarocks` shares the shape without the requirement.
        unpinned: Option<String>,
    },
}

impl VersionPin {
    /// A version that follows the name and is optional — the common case.
    pub fn after(args: Vec<String>) -> Self {
        VersionPin::After {
            args,
            unpinned: None,
        }
    }

    /// A version that follows the name and without which the manager will not install.
    pub fn after_required(args: Vec<String>, unpinned: &str) -> Self {
        VersionPin::After {
            args,
            unpinned: Some(unpinned.to_string()),
        }
    }

    /// Produce the install argument(s) for `name` pinned to `version`.
    pub fn apply(&self, name: &str, version: &str) -> Vec<String> {
        match self {
            VersionPin::Inline(tmpl) => {
                vec![tmpl.replace("{name}", name).replace("{version}", version)]
            }
            // The args go into the subcommand args, not beside the name — see the caller.
            VersionPin::Before(_) => vec![name.to_string()],
            VersionPin::After { args, .. } => {
                let mut out = vec![name.to_string()];
                out.extend(
                    args.iter()
                        .map(|f| f.replace("{name}", name).replace("{version}", version)),
                );
                out
            }
        }
    }

    /// The args this pin contributes *before* the name, if any.
    pub(crate) fn leading_args(&self, version: &str) -> Vec<String> {
        match self {
            VersionPin::Before(args) => args
                .iter()
                .map(|f| f.replace("{version}", version))
                .collect(),
            _ => Vec::new(),
        }
    }

    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    /// Whether this pin puts an **option** after the name.
    ///
    /// The one shape `--` cannot precede: behind a terminator, gem's `-v` stops being an
    /// option and becomes a gem name. Answered by looking at the token rather than by asking
    /// the backend author to declare it, because `-` is what "option" means to every argument
    /// parser and a declaration can be wrong.
    pub fn emits_trailing_option(&self) -> bool {
        match self {
            VersionPin::After { args, .. } => args.first().is_some_and(|a| a.starts_with('-')),
            _ => false,
        }
    }

    /// Whether this pin forbids batching several specs into one command.
    pub(crate) fn is_one_per_command(&self) -> bool {
        matches!(self, VersionPin::Before(_))
    }

    /// What this manager should be asked for when the declaration names no version.
    pub fn unpinned(&self) -> Option<&str> {
        match self {
            VersionPin::After {
                unpinned: Some(u), ..
            } => Some(u),
            _ => None,
        }
    }
}

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so a private item is invisible there.
/// True when a version string represents a real pin (not "latest"/"*"/empty).
pub fn is_concrete_version(v: &str) -> bool {
    !v.is_empty() && v != "latest" && v != "*"
}

/// How a backend answers "which packages did the user actually ask for?" — the question
/// `adopt` must get right before it adopts anything into managed state.
///
/// This is stated per backend rather than inferred from the absence of config: "no manual
/// command configured" is ambiguous between *"listing everything is the correct answer"*
/// (winget has no dependencies) and *"we have no idea"* (pip does). Conflating the two is
/// how an entire dependency graph gets adopted and then purged.
#[derive(Debug, Clone)]
pub enum ManualListing {
    /// Every installed package was user-requested *and* the manager can reinstall all of
    /// them, so `list_installed` *is* the manual set (choco, mas, dotnet).
    ///
    /// Both halves are load-bearing. winget satisfies the first and fails the second, which
    /// is why it is [`ExportFile`](Self::ExportFile) and not this.
    AllInstalled,
    /// The manager reports its explicit set via a command of its own.
    Command {
        /// Binary to run, when it is neither the backend nor `list_binary` (apt's manual
        /// set lives in `apt-mark`, a third binary distinct from its `dpkg-query` lister).
        /// `None` falls back to `list_binary`, then the backend name.
        binary: Option<String>,
        args: Vec<String>,
        format: ManualFormat,
    },
    /// The manager writes its own restorable set to a file it is handed.
    ///
    /// **The only variant that answers "what could I put back" rather than "what is here".**
    /// Every other one reads a listing of the machine; a manager whose listing includes things
    /// it cannot reinstall needs this, because adoption's whole output is declarations that
    /// have to converge later. On winget the two sets differ by 186 of 280 rows.
    ExportFile {
        /// `None` falls back to `list_binary`, then the backend name.
        binary: Option<String>,
        /// `{file}` is replaced with a path in a directory Shall owns for the call. The
        /// manager writes there; nothing reads its stdout for the set.
        args: Vec<String>,
        format: ExportFormat,
    },
    /// The manager installs dependencies but exposes no way to tell them apart from what
    /// the user chose (pip, gem, zypper, pkgin). Adoption must skip the backend entirely.
    Unsupported,
}

/// The shape of a `ManualListing::Command`'s output.
#[derive(Clone)]
pub enum ManualFormat {
    /// Same shape as `list_args` output — reuse the backend's installed parser.
    SameAsInstalled,
    /// One bare package name per line, no versions (`apt-mark showmanual`).
    BareNames,
    /// A shape of its own, read by its own function.
    ///
    /// **The manual set is a different question from the installed set, so it may have a
    /// different answer shape.** conda's is `conda env export --from-history --json`, whose
    /// `dependencies` array holds match-specs (`python=3.13`) rather than the package objects
    /// `conda list --json` returns — the same manager, two formats, and no amount of leniency
    /// in one parser should be asked to cover both. That leniency is what `Q40` was.
    Read(InstalledReader),
}

impl std::fmt::Debug for ManualFormat {
    /// A reader is a closure with no useful rendering, and this type is printed by
    /// `ManagerConfig`'s `Debug` — which `resolve_settings` scans for unresolved placeholders.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SameAsInstalled => f.write_str("SameAsInstalled"),
            Self::BareNames => f.write_str("BareNames"),
            Self::Read(_) => f.write_str("Read(..)"),
        }
    }
}

/// The shape of the file a [`ManualListing::ExportFile`] manager writes.
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    /// `winget export`'s JSON: `Sources[].Packages[].PackageIdentifier`.
    WingetJson,
}

impl std::fmt::Debug for MachineListing {
    /// The reader is a closure and has no useful rendering; the argv is what identifies this.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MachineListing")
            .field("binary", &self.binary)
            .field("args", &self.args)
            .finish_non_exhaustive()
    }
}

/// The one command that names everything this manager has an update for (`Q44`).
///
/// **A different question from `search`, not a faster route to the same answer.** Without this,
/// `list --outdated` asks each manager about one package at a time — and `Searchable::lookup`
/// defaults to a whole search for that one name, so a machine with 280 packages ran 280
/// registry searches. Measured: 771.4s, against 2.9s for the plain listing that fed it.
/// One command an install would run, before it is run.
///
/// **Not an argv alone.** `unverified` is the fact the *runner* needs to translate a failure
/// into advice ("helm 3 cannot verify plugins"), and it is decided while the argv is built — so
/// a plain `(String, Vec<String>)` would either lose it or make the runner recompute it from
/// the specs a second time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedInstall {
    pub binary: String,
    pub args: Vec<String>,
    /// Whether the specs asked for something this manager's own verification covers.
    pub unverified: bool,
}

#[derive(Clone)]
pub struct OutdatedProbe {
    /// `None` falls back to `list_binary`, then the backend name.
    pub binary: Option<String>,
    pub args: Vec<String>,
    /// Reads the manager's answer into `(name, version available)` pairs. The installed
    /// version is not taken from here — the caller already has it, and a manager that reports
    /// its own idea of "current" has been wrong about it before.
    pub parse: PackageReader,
    /// This manager says *"nothing is out of date"* by exiting non-zero with nothing on either
    /// stream, and documents that exit as the answer.
    ///
    /// `pacman -Qu` is the one. That shape is exactly what `Q40` calls a failed read —
    /// correctly in general, and wrongly here — so it is translated back for the manager whose
    /// meaning is known, rather than by loosening the rule for every read in the program. A
    /// pacman that genuinely failed says so on stderr and still raises.
    pub silence_is_none: bool,
}

/// A machine-readable listing this manager *may* support, and how to read it (`Q43`).
///
/// **Asked for, not assumed.** Every one of these is a flag that arrived in some version of the
/// tool — `dotnet tool list --format json` needs SDK 10, `pixi global list --json` a recent
/// pixi — and Shall does not control which version is installed. Passing an unsupported flag
/// makes the command fail with a usage message, and a caller that reads that as the listing has
/// reproduced `Q40`: a manager silently reporting an empty machine, for users on older tooling
/// only, who are the least likely to notice.
///
/// So it is a negotiation. Ask; if the manager refuses, use [`ManagerConfig::list_args`] and say
/// so at `debug`. It costs one failed invocation on old tooling and nothing on current, and it
/// needs no version table to go stale. The listing memo makes "once per run" free — a run asks
/// each manager for its listing exactly once either way.
/// Reads a manager's output into packages.
///
/// An `Arc<dyn Fn>` rather than a `fn` pointer because a custom backend's parser is a runtime
/// `ParserSpec` it has to close over, and the onboarder's whole claim is that a custom backend
/// is a first-class peer of a built-in (U2). A `fn` pointer would have made these two fields
/// the exception.
pub type PackageReader = std::sync::Arc<dyn Fn(&str) -> Vec<Package> + Send + Sync>;

/// Reads a manager's output into the packages it says are **installed**, or admits it did not
/// recognise the bytes.
///
/// Separate from [`PackageReader`] because the two answer different questions and only one of
/// them is dangerous when it comes back empty. An empty *outdated* list means nothing needs
/// upgrading, which is the common case and a fact the caller can act on. An empty *installed*
/// list means the machine is bare, which the planner answers by installing every declaration
/// and dropping every removal — so a listing nobody could parse must not be able to spell
/// itself that way. See [`crate::parsers::Unrecognised`].
pub type InstalledReader =
    std::sync::Arc<dyn Fn(&str) -> crate::parsers::ParseResult + Send + Sync>;

#[derive(Clone)]
pub struct MachineListing {
    /// `None` falls back to `list_binary`, then the backend name.
    pub binary: Option<String>,
    pub args: Vec<String>,
    /// Reads what those args produce. A *different* function from the text parser, never the
    /// same one made lenient: one parser that accepts two shapes is how a malformed answer in
    /// one of them gets silently read as the other.
    ///
    /// Fallible for the same reason the text parser is. This path is *more* exposed, not less:
    /// it exists because a flag may or may not be present in the installed version of the tool,
    /// so it is the one listing whose shape Shall has already admitted it cannot predict.
    pub parse: InstalledReader,
}

/// Reads a manager's output into bare names — dependencies, and anything else that is a list
/// of names rather than of packages.
pub type NameReader = std::sync::Arc<dyn Fn(&str) -> Vec<String> + Send + Sync>;

/// How this manager empties its download cache (`X.3` levels 1–2).
///
/// **`None` is a claim that the manager has no cache verb, not that nobody wrote one.** Until
/// 2026-08-06 it was neither: `ManagerConfig` had no field at all and `GenericUpgradable` did
/// not implement `clean_cache`, so every one of the forty data-path backends answered
/// `Unsupported`. `handle_clean_cache` filters that out silently, so `shall clean-cache` on a
/// Debian machine printed *"No backend on this machine has a cache to clear"* while
/// `/var/cache/apt/archives` sat there. Six hand-written modules had the verb; the shared
/// machinery could not express it, which is the terminator split running the other way.
#[derive(Debug, Clone)]
pub struct CacheClean {
    /// `None` falls back to the backend's own binary. Void empties its cache with
    /// `xbps-remove`, which is its *remover*, not its installer.
    pub binary: Option<String>,
    pub args: Vec<String>,
}

/// How this manager answers "what does this package need?" — **reported, never planned from**.
///
/// A probe rather than a bare argv list because the answer's shape is the manager's: dnf prints
/// one bare name per line, pacman prints several on one labelled row, apt prints one per
/// labelled line. One parser made lenient enough for all three is one parser that reads a
/// malformed answer in one shape as a valid answer in another.
#[derive(Clone)]
pub struct DependsProbe {
    /// `None` falls back to the backend's own binary. Void asks `xbps-query`, which is neither
    /// the installer it is named for nor its remover.
    pub binary: Option<String>,
    /// `{name}` is the package asked about.
    pub args: Vec<String>,
    pub parse: NameReader,
}

impl std::fmt::Debug for DependsProbe {
    /// The reader is a closure and has no useful rendering; the argv is what identifies this.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DependsProbe")
            .field("binary", &self.binary)
            .field("args", &self.args)
            .finish_non_exhaustive()
    }
}

/// What a manager's repository listing prints.
///
/// Stated per backend rather than guessed from the output, because the two shapes are not
/// distinguishable from one line: a bare name and a `name url` row whose url happens to be
/// missing look identical, and reading the first as the second silently reports every
/// repository as having no source.
#[derive(Debug, Clone)]
pub enum RepoListing {
    /// One row per repository, whose first two whitespace columns are the name and the source.
    /// Every manager with a `repo list` verb of its own.
    Columns,
    /// Bare names, one per line, and the source is a second question about one of them.
    /// `{name}` is the repository.
    ///
    /// pacman is the one: `pacman-conf --repo-list` prints names and nothing else, and
    /// `pacman-conf -r <name> Server` prints that one repository's mirror.
    NamesThenDetail(Vec<String>),
}

/// A dry run of the manager's own orphan verb, and how to read the names back out of it.
///
/// `apt-get autoremove --dry-run` prints `Remv libfoo1 [1.2-3]` per package; the prefix is
/// what separates those lines from the summary counts and the "0 upgraded" line.
#[derive(Debug, Clone)]
pub struct OrphanDryRun {
    /// Binary to run, when it is not the backend's own (apt's autoremove is `apt-get`).
    pub binary: Option<String>,
    pub args: Vec<String>,
    pub removes_line_prefix: String,
}
