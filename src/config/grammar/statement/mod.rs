//! The grammar's types: what a line can mean once it has been read.
//!
//! The parser that produces them is [`parse`]'s module and the rules a produced line is
//! held to are [`validate`]'s; this file holds neither, so "what does a statement *be*"
//! is answerable without reading 2,500 lines.

use super::error::{GrammarError, Origin, Result};
use super::options::{parse_short, Options};

/// `re` is reserved: `apt:re:^fonts-` must always mean a regex, so a custom backend named
/// `re` (which the onboarder would otherwise happily accept) would make `re:foo`
/// ambiguous forever. `list` is reserved for the same reason — it names the `priority` file
/// inside a backend chain (`apt,list:rg`), so a backend called `list` would make that
/// unreadable.
pub const RESERVED_BACKEND_NAMES: &[&str] = &["re", "list"];

/// The word that means "the `priority` file" where a backend name is expected.
pub const PRIORITY_KEYWORD: &str = "list";

/// Which managers a line will accept, when it has not pinned exactly one.
///
/// A pin (`apt:rg`) says apt or nothing — carried in `PackageDecl::backend`, so this only
/// describes the unpinned case. Separating the two is what lets `apt:rg` keep meaning apt on
/// a machine that also has dnf, while `apt,dnf:rg` and a bare `rg` stay installable on a
/// machine that has neither apt nor the manager some other machine froze the name to.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Candidates {
    /// A bare name (`rg`), spelled explicitly as `list:rg`: every manager in `priority`, in
    /// that order.
    #[default]
    Priority,
    /// `apt,dnf:rg` — these, in this order, and nothing else.
    Named(Vec<String>),
    /// `apt,list:rg` — these first, then the rest of `priority` in its own order.
    NamedThenPriority(Vec<String>),
}

impl Candidates {
    /// The managers to ask, in order. `priority` supplies the tail for the two variants that
    /// end in `list`; a name already asked for is not asked twice.
    pub fn order(&self, priority: &[String]) -> Vec<String> {
        let (head, tail): (&[String], &[String]) = match self {
            Candidates::Priority => (&[], priority),
            Candidates::Named(names) => (names, &[]),
            Candidates::NamedThenPriority(names) => (names, priority),
        };
        let mut out: Vec<String> = head.to_vec();
        for name in tail {
            if !out.contains(name) {
                out.push(name.clone());
            }
        }
        out
    }

    /// Whether this line would accept `backend`. A lock naming a manager the line no longer
    /// lists is not an answer to the question the line is now asking.
    pub fn accepts(&self, backend: &str, priority: &[String]) -> bool {
        self.order(priority).iter().any(|b| b == backend)
    }
}

/// What a package line selects inside its backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    Name(String),
    /// `BACKEND:re:PATTERN` — matches names in that backend. Live by default; frozen only
    /// when `locks/` holds an entry for it (II.15).
    Regex(String),
}

impl Selector {
    pub fn as_str(&self) -> &str {
        match self {
            Selector::Name(n) | Selector::Regex(n) => n,
        }
    }
}

/// A package declaration: the backend (or none, meaning "resolve via `priority`"), what it
/// selects, and its options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDecl {
    /// `Some` only when the line pinned exactly one manager (`apt:rg`). II.7 resolves the
    /// rest through `candidates`, then locks the answer — the unpinned name is the question,
    /// the lock is the answer (V.16).
    pub backend: Option<String>,
    /// Which managers may answer, when `backend` is `None`. Ignored when it is `Some`.
    pub candidates: Candidates,
    pub selector: Selector,
    pub options: Options,
}

/// A reference to a module (lowercase) or a profile (Capitalized). Case is what tells them
/// apart, so `(Work | gaming) & security` reads without extra syntax (II.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    Module(String),
    Profile(String),
}

impl Reference {
    /// Classify by the first character's case. A name starting with neither (a digit,
    /// `_`) is rejected by the caller rather than guessed at.
    pub fn classify(name: &str) -> Option<Self> {
        let first = name.chars().next()?;
        if first.is_uppercase() {
            Some(Reference::Profile(name.to_string()))
        } else if first.is_lowercase() {
            Some(Reference::Module(name.to_string()))
        } else {
            None
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Reference::Module(n) | Reference::Profile(n) => n,
        }
    }
}

/// What a declaration declares — the keyword half of a statement, as a type.
///
/// **This exists because two dispatches over the extras kinds were written as `match kind: &str`
/// with a catch-all, and both catch-alls were wrong in a way no reader could see.** The teardown
/// answered `Ok(())` for a kind it did not recognise, which reports the undo as *done* to a
/// caller that then clears the ledger row — so the resource is forgotten while still in effect
/// and no later sync looks at it again. The other answered `None`, which means *unverifiable*,
/// which places: a kind falling through there is re-applied on every sync for ever.
///
/// A `&str` cannot be matched exhaustively, so both dispatches had a branch nobody had to
/// justify. This can, and neither has one now: a keyword added to the grammar does not compile
/// until the teardown says how to undo it and the probe says whether it can be checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceKind {
    Repo,
    Shim,
    Schedule,
    Service,
    Link,
    Setting,
    Exec,
    Generate,
    Dotfiles,
    Firewall,
    Dir,
}

impl ResourceKind {
    /// The keyword a user writes, which is also the prefix of every key of this kind.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Repo => "repo",
            Self::Shim => "shim",
            Self::Schedule => "schedule",
            Self::Service => "service",
            Self::Link => "link",
            Self::Setting => "setting",
            Self::Exec => "exec",
            Self::Generate => "generate",
            Self::Dotfiles => "dotfiles",
            Self::Firewall => "firewall",
            Self::Dir => "dir",
        }
    }

    /// Every kind, in declaration order — so a test or a report can enumerate them rather than
    /// restate the list.
    pub const ALL: &'static [ResourceKind] = &[
        Self::Repo,
        Self::Shim,
        Self::Schedule,
        Self::Service,
        Self::Link,
        Self::Setting,
        Self::Exec,
        Self::Generate,
        Self::Dotfiles,
        Self::Firewall,
        Self::Dir,
    ];
}

impl std::str::FromStr for ResourceKind {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, ()> {
        Self::ALL
            .iter()
            .copied()
            .find(|k| k.as_str() == s)
            .ok_or(())
    }
}

impl std::fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One parsed line. Covers every statement kind the grammar accepts: II.2's declarations
/// and typed lines (`Package`, `Absent`, `Repo`, `Shim`, `Schedule`, `Service`, `Link`,
/// `Use`) **and** II.4's set operations (`Exclude`, `Intersect`, `Subtract`, `Expr`) — the
/// latter belong to the set-math grammar, not II.2's statement table, so this is not "II.2's
/// full list" but the union of the two grammars a module line can be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Package(PackageDecl),
    /// `absent:BACKEND:NAME` — declare it must not exist. The one thing Shall may remove
    /// that it does not manage, because you named it (V.7).
    Absent(PackageDecl),
    /// `repo:BACKEND:SPEC` — a repository, for a named backend (V.47). A PPA is apt's, a
    /// COPR dnf's; guessing the backend runs the wrong system command, so it is named.
    Repo {
        backend: String,
        spec: String,
    },
    Shim(String, Options),
    Schedule(String, Options),
    Service(String, Options),
    Link(String, Options),
    /// `setting:SCHEMA/KEY @value=…` — a desktop setting whose home is a settings store
    /// rather than a file (X.4). GNOME and KDE keep configuration in dconf and kconfig, so
    /// `link:` cannot reach it; the adapter is chosen by what is running, not by what was
    /// typed, which is why this is a statement and not a backend.
    Setting(String, Options),
    /// `exec:PATH @runs=N` — run a script the config carries (XIII.3). A *verb*, not a noun:
    /// its `when` decides whether the machine wants it, and `locks/exec.toml` (keyed by the
    /// script's content hash) decides whether it already happened. Unlike every other
    /// statement, a false `when` does not mean "undo" — a script that succeeds makes its own
    /// condition false, so treating false as removal would flap. See the three-state table in
    /// XIII.3. The script goes through II.12's approval ledger like any other code the repo
    /// runs ("hash everything, no exceptions").
    Exec(String, Options),
    /// `generate:PATH` — run a command whose stdout is *declarations*, merged into the desired
    /// state as if typed (XIII.30, U33). The dangerous half of Lisp: a config that computes its
    /// state rather than stating it. **Off by default** (`allow_generators`), it runs through the
    /// II.12 ledger like `exec:`, its output passes the same guard and removal preview as a typed
    /// line, and a generator that fails is a failed sync — never a silently empty declaration set,
    /// which is a mass-removal input (VI.0). Takes no options: it runs every resolution to
    /// compute the current answer, so there is no `@runs` ceiling to set.
    Generate(String, Options),
    /// `dotfiles:PATH` — a folder mirrored into place, one file at a time (XIII.21).
    ///
    /// Every other statement names one thing; this names a tree and stands for as many
    /// declarations as it holds. It links **files**, never directories (U22): a symlinked
    /// directory takes everything the application later writes there into the git-tracked
    /// repo, and `bundle` then hands it to whoever the backup goes to.
    Dotfiles(String, Options),
    /// `firewall:22/tcp`, `firewall:default/incoming @value=deny` — a declared perimeter
    /// (Part XI). One spelling across ufw, firewalld and Windows Defender, which is the whole
    /// argument for a built-in backend rather than a per-machine `[[backend]]` naming `ufw`.
    Firewall(String, Options),
    /// `dir:PATH` — an idempotent directory (U71). Creates the directory (and parents)
    /// if absent, sets ownership and permissions. Teardown removes only if empty.
    Dir(String, Options),
    /// `invariant:NAME @path=…,backed-by=ADAPTER:TARGET,max-age=…` — a promise about this
    /// machine that no sync can make true (#74).
    ///
    /// Not a resource: nothing is placed, so nothing is undone and nothing reaches the extras
    /// ledger. What the line buys is a question `check` asks of the backup tool's own listing —
    /// does a copy of the path exist elsewhere, and is the newest one younger than `@max-age`?
    Invariant(String, Options),
    /// `use editors` / `use workstation(user=shaul, gpu=nvidia)` — bring in a module or profile
    /// (II.2), optionally with **arguments** binding that module's `param`s (U32). The args are
    /// empty for the ordinary form; a profile referenced with args is refused at parse time,
    /// because a profile has no parameters to bind.
    Use(Reference, Vec<(String, String)>),
    /// `param NAME` / `param NAME = DEFAULT` — a module parameter (U32). A `param` with no
    /// default is required: a `use` that omits it is a loud error naming the module and the
    /// parameter, never an empty string that makes a `when` silently false (V.78). Legal only in
    /// a module; parsed here so there is one parser, and rejected by file context like `schedule:`.
    Param {
        name: String,
        default: Option<String>,
    },
    /// `exclude heavy` — subtract that module's or profile's packages (II.4).
    Exclude(Reference),
    /// `intersect security` — keep only packages that are also in it (II.4).
    Intersect(Reference),
    /// `-vim` — subtract one package (II.4).
    ///
    /// Not an infix operator: real package names contain `-` (`g++` aside, `fonts-noto`
    /// does), so `a - b` cannot be told from a package called `a - b` without quoting, and
    /// there are no quotes (V.10).
    Subtract(String),
    /// `(Work | gaming) & security` — a set expression over modules and profiles (II.4).
    Expr(String),
    /// `NAME = VALUE` — a variable (IX.2). Legal only in the `vars` file; parsed here so
    /// there is one parser, and rejected by file context the way `schedule:` is.
    Var {
        name: String,
        value: String,
    },
}

impl Statement {
    /// How this statement is named: `service:nginx`, `apt:jq`, `use work`, `-vim`.
    ///
    /// **One spelling, because it had three.** Set math keyed statements one way, `edit`'s
    /// line matcher a second, the teardown ledger a third — three lists of the same twelve
    /// variants, each of which had to be extended whenever a statement kind was added, and
    /// none of which the compiler could check against the others. A statement's identity is a
    /// property of the statement, so it lives on the statement.
    ///
    /// Written form, not resolved form: a bare `jq` keys as `jq`, because set math runs while
    /// the files are being read and nothing has probed a backend yet.
    pub fn key(&self) -> String {
        match self {
            Statement::Package(d) | Statement::Absent(d) => match &d.backend {
                Some(b) => format!("{}:{}", b, d.selector.as_str()),
                None => d.selector.as_str().to_string(),
            },
            Statement::Repo { backend, spec } => format!("repo:{}:{}", backend, spec),
            Statement::Shim(n, _) => format!("shim:{}", n),
            Statement::Schedule(n, _) => format!("schedule:{}", n),
            Statement::Service(n, _) => format!("service:{}", n),
            Statement::Link(n, _) => format!("link:{}", n),
            Statement::Setting(n, _) => format!("setting:{}", n),
            Statement::Exec(n, _) => format!("exec:{}", n),
            Statement::Generate(n, _) => format!("generate:{}", n),
            Statement::Dotfiles(n, _) => format!("dotfiles:{}", n),
            Statement::Firewall(n, _) => format!("firewall:{}", n),
            Statement::Dir(n, _) => format!("dir:{}", n),
            Statement::Invariant(n, _) => format!("invariant:{}", n),
            Statement::Use(r, _) => format!("use {}", r.name()),
            Statement::Param { name, .. } => format!("param {}", name),
            Statement::Exclude(r) => format!("exclude {}", r.name()),
            Statement::Intersect(r) => format!("intersect {}", r.name()),
            Statement::Subtract(p) => format!("-{}", p),
            Statement::Var { name, .. } => format!("{} =", name),
            Statement::Expr(e) => e.clone(),
        }
    }

    /// The keyword that introduces this statement — `service`, `link`, `firewall` — for the
    /// kinds that have one.
    ///
    /// `None` for a package line (whose prefix is a *backend*, not a keyword) and for set math
    /// (an operation, not a thing). A caller that wants to group or filter by kind asks here
    /// rather than re-splitting [`key`](Self::key) on `:`, which would read `apt:jq` as the
    /// kind `apt`.
    pub fn kind(&self) -> Option<ResourceKind> {
        Some(match self {
            Statement::Repo { .. } => ResourceKind::Repo,
            Statement::Shim(..) => ResourceKind::Shim,
            Statement::Schedule(..) => ResourceKind::Schedule,
            Statement::Service(..) => ResourceKind::Service,
            Statement::Link(..) => ResourceKind::Link,
            Statement::Setting(..) => ResourceKind::Setting,
            Statement::Exec(..) => ResourceKind::Exec,
            Statement::Generate(..) => ResourceKind::Generate,
            Statement::Dotfiles(..) => ResourceKind::Dotfiles,
            Statement::Firewall(..) => ResourceKind::Firewall,
            Statement::Dir(..) => ResourceKind::Dir,
            Statement::Package(_)
            | Statement::Absent(_)
            | Statement::Invariant(..)
            | Statement::Use(..)
            | Statement::Param { .. }
            | Statement::Exclude(_)
            | Statement::Intersect(_)
            | Statement::Subtract(_)
            | Statement::Expr(_)
            | Statement::Var { .. } => return None,
        })
    }

    pub fn options(&self) -> Option<&Options> {
        match self {
            Statement::Shim(_, opts)
            | Statement::Schedule(_, opts)
            | Statement::Service(_, opts)
            | Statement::Link(_, opts)
            | Statement::Setting(_, opts)
            | Statement::Exec(_, opts)
            | Statement::Generate(_, opts)
            | Statement::Dotfiles(_, opts)
            | Statement::Firewall(_, opts)
            | Statement::Dir(_, opts)
            | Statement::Invariant(_, opts) => Some(opts),
            _ => None,
        }
    }

    /// Whether an export should carry this line (II.94).
    ///
    /// **Asked of the statement, not read at the export site.** `@export=no` is validated here,
    /// so the answer to "was this line opted out" has exactly one implementation — the exporter
    /// asks, and a statement kind added tomorrow inherits the behaviour by existing.
    pub fn exports(&self) -> bool {
        let options = match self {
            Statement::Package(decl) | Statement::Absent(decl) => &decl.options,
            other => match other.options() {
                Some(options) => options,
                None => return true,
            },
        };
        options.one(EXPORT_KEY) != Some("no")
    }

    /// What this statement names, without its keyword: `nginx` for `service:nginx`.
    ///
    /// The `key` minus the `kind`, so the two can never disagree about where the boundary is.
    pub fn subject(&self) -> Option<String> {
        let kind = self.kind()?;
        let key = self.key();
        Some(
            key.strip_prefix(kind.as_str())?
                .trim_start_matches(':')
                .to_string(),
        )
    }

    /// When in a sync this statement's work happens (II.7).
    ///
    /// Exhaustive on purpose: a statement kind added to the grammar cannot compile until
    /// somebody has said where in the sync it belongs, which is the one question every one of
    /// the four misses below failed to ask.
    pub fn phase(&self) -> Phase {
        match self {
            Statement::Repo { .. } => Phase::Repositories,
            Statement::Package(_) | Statement::Absent(_) => Phase::Packages,
            // A `shim:` wraps a tool that must already be installed, a `service:` enables a
            // unit a package just laid down, a `link:` writes a config a package expects, and
            // a `setting:` addresses a store a package provides. Each leans on the package
            // plan having run, which is what makes them dependents and not phase 2.
            Statement::Shim(..)
            | Statement::Service(..)
            | Statement::Link(..)
            | Statement::Setting(..)
            | Statement::Dir(..) => Phase::Dependents,
            Statement::Dotfiles(..) => Phase::Dotfiles,
            Statement::Firewall(..) => Phase::Firewall,
            Statement::Schedule(..) => Phase::Schedules,
            Statement::Exec(..) => Phase::Execs,
            // A `generate:` runs before the desired state exists and is replaced by the
            // declarations it prints, so nothing downstream ever sees one. Set math, a
            // `param`, a `use` and a variable are consumed the same way: they say what to
            // read, never what to put on the machine.
            // An `invariant:` is the one member of this group that survives resolution — it is
            // carried to `check`, which reads it. It still has no phase work: a sync cannot make
            // a backup exist, and pretending otherwise would put a 3am `restic backup` inside
            // every `sync` — a decision, and one this grammar does not make.
            Statement::Generate(..)
            | Statement::Use(..)
            | Statement::Param { .. }
            | Statement::Exclude(_)
            | Statement::Intersect(_)
            | Statement::Subtract(_)
            | Statement::Expr(_)
            | Statement::Var { .. }
            | Statement::Invariant(..) => Phase::Resolution,
        }
    }
}

/// Where in a sync a statement's work happens (II.7).
///
/// **The order of a sync was a comment, and membership of it was a chain of `||`.** Which
/// phase a kind belonged to was written down in places that could not check each other: the
/// dispatch list in `sync`, the dry-run branch's copy of it, `DesiredState`'s per-kind
/// accessors, and `has_non_package_work`'s chain of ors. Every statement kind added since was
/// missed by one of them — extras, then `exec:`, then `dotfiles:`, then `firewall:`. A phase
/// is a property of the statement, so it lives on the statement, and "is there work after the
/// packages?" becomes a comparison instead of a list somebody has to remember to extend.
///
/// **The variants are declared in the order they run, and `Ord` is that order.** `phase >
/// Phase::Packages` is exactly "work the package transaction does not cover", which is the
/// question the chain of ors was written to answer and the one it got wrong four times.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Phase {
    /// Not work. Consumed while the desired state is being computed — set math, a `param`, a
    /// `use`, a variable, a `generate:`. None of these survives into `DesiredState::extras`,
    /// so nothing downstream can be handed one. An `invariant:` is here for the same reason —
    /// it is a fact, and there is no work — but it is carried in `DesiredState::invariants`
    /// rather than dropped, because `check` reads it.
    Resolution,
    /// Phase 1 — `repo:`, before the packages. A package from a PPA cannot install until the
    /// PPA is there.
    Repositories,
    /// Phase 2 — the package transaction: `apt:jq`, `absent:apt:nano`.
    Packages,
    /// Phase 3 — the extras that lean on a package: `shim:`, `service:`, `link:`, `setting:`.
    Dependents,
    /// Phase 3b (7n) — `dotfiles:`, a tree standing for the `link:` lines it holds.
    Dotfiles,
    /// Phase 3c (Part XI) — `firewall:`. After the packages, because a rule usually exists to
    /// let in something that was just installed.
    Firewall,
    /// Phase 4 (S21) — `schedule:`, provisioned onto the OS scheduler.
    Schedules,
    /// Phase 4b (XIII.3) — `exec:`, after the packages and dependents a script leans on.
    Execs,
}

impl Phase {
    /// The phase that runs after this one, or `None` at the end of a sync.
    ///
    /// **The order lives here and only here.** A phase added to the enum cannot compile
    /// without being given a successor, and [`all`](Self::all) walks this chain rather than
    /// reading a second list — so there is no hand-copied ordering to fall out of step with
    /// the one the dispatch iterates.
    pub fn next(self) -> Option<Phase> {
        Some(match self {
            Phase::Resolution => Phase::Repositories,
            Phase::Repositories => Phase::Packages,
            Phase::Packages => Phase::Dependents,
            Phase::Dependents => Phase::Dotfiles,
            Phase::Dotfiles => Phase::Firewall,
            Phase::Firewall => Phase::Schedules,
            Phase::Schedules => Phase::Execs,
            Phase::Execs => return None,
        })
    }

    /// Every phase, in the order a sync runs them.
    pub fn all() -> impl Iterator<Item = Phase> {
        std::iter::successors(Some(Phase::Resolution), |p| p.next())
    }

    /// The phases whose work happens after the package transaction has closed — the list
    /// `sync` dispatches once the engine has returned.
    pub fn after_packages() -> impl Iterator<Item = Phase> {
        Phase::all().filter(|p| *p > Phase::Packages)
    }
}

/// Decides whether a `prefix:` names a real backend. Injected rather than hardcoded: the
/// answer is host-dependent (there is no `winget` on Linux) and the onboarder can add
/// backends at runtime, so a static list would be a second copy of a fact the registry
/// already owns (P4).
pub trait BackendNames {
    fn is_backend(&self, name: &str) -> bool;

    /// The backends a group name expands to (U18), or `None` when it is not a group. A group is
    /// a shorthand for a comma-chain, so expansion happens here, in the one parser, and the
    /// expanded members go through the same backend check every chain part does.
    ///
    /// Default: nothing is a group. The paths with no `groups` file — and every test that only
    /// cares about backends — keep working unchanged.
    fn expand_group(&self, _name: &str) -> Option<Vec<String>> {
        None
    }
}

impl<F: Fn(&str) -> bool> BackendNames for F {
    fn is_backend(&self, name: &str) -> bool {
        self(name)
    }
}

/// What a reserved word *is*, as opposed to how it is spelled.
///
/// [`Foreign`](KeywordRole::Foreign) is the distinction the table could not previously make:
/// `use` and `if` were both "no colon, no `build`", so nothing could tell a directive this
/// grammar has from a word it deliberately refuses. Anything quantifying over the language —
/// the spec ratchet in `tests/review/grammar_table_matches_the_spec_tests.rs` above all — would then
/// have to re-derive the split from a second list, which is how the three copies this table
/// replaced went out of step in the first place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeywordRole {
    /// Introduces a typed statement and is written `word:` — `link:`, `absent:`.
    Prefix,
    /// A directive of this grammar, written bare — `use`, `when`, `param`.
    Directive,
    /// Not this grammar's word at all. Reserved only so that arriving from another config
    /// language does not install a package: `gem:if` and `npm:else` are real packages.
    Foreign,
}

/// A word this grammar reserves: the statement it introduces, and the form to write instead
/// when it turns up alone on a line (Q16).
pub struct Keyword {
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    /// How the word is written when it introduces its statement: `link:` for a resource
    /// prefix, `when` for a directive. The bare word is this with the colon trimmed, which is
    /// why there is one field and not two that can disagree.
    pub spelling: &'static str,
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    /// The form the user probably meant, shown in the refusal.
    pub means: &'static str,
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    pub role: KeywordRole,
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    /// Builds the statement from `(name, options)`. `None` for the prefixes parsed above the
    /// dispatch loop (`absent:`, `repo:` have payloads of their own shape) and for the
    /// directives, which are not resources at all.
    pub build: Option<fn(String, Options) -> Statement>,
}

impl Keyword {
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    /// The word alone — `link:` written as the user half-typed it.
    pub fn word(&self) -> &'static str {
        self.spelling.trim_end_matches(':')
    }

    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    /// Whether the word is written `word:` — a resource prefix rather than a directive.
    ///
    /// Asks the role, not the spelling: the colon is how a prefix is *written* and the role is
    /// what it *is*. `keyword_roles_and_spellings_agree` holds the two together.
    pub fn takes_colon(&self) -> bool {
        matches!(self.role, KeywordRole::Prefix)
    }
}

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so a private item is invisible there.
/// Every keyword that introduces a statement, in one list.
///
/// This is the list the dispatch loop below iterates, the list the "unrecognised line" error
/// names, and the list a bare word is checked against. Three separate copies existed and had
/// drifted apart — the error message knew six prefixes, the dispatcher nine, and the
/// set-expression guard a *different* nine, so `setting:HKCU\Software\Foo` was read as a set
/// difference by the only one of the three that had never heard of `setting:`.
pub const KEYWORDS: &[Keyword] = &[
    Keyword {
        spelling: "absent:",
        means: "absent:apt:libreoffice",
        role: KeywordRole::Prefix,
        build: None,
    },
    Keyword {
        spelling: "repo:",
        means: "repo:apt:ppa:deadsnakes/ppa",
        role: KeywordRole::Prefix,
        build: None,
    },
    Keyword {
        spelling: "shim:",
        means: "shim:NAME @source=/path/to/binary",
        role: KeywordRole::Prefix,
        build: Some(Statement::Shim),
    },
    Keyword {
        spelling: "schedule:",
        means: "schedule:nightly@cron=@daily,run=sync",
        role: KeywordRole::Prefix,
        build: Some(Statement::Schedule),
    },
    Keyword {
        spelling: "service:",
        means: "service:nginx @status=running",
        role: KeywordRole::Prefix,
        build: Some(Statement::Service),
    },
    Keyword {
        spelling: "link:",
        means: "link:/path/to/source @target=/path/to/destination",
        role: KeywordRole::Prefix,
        build: Some(Statement::Link),
    },
    Keyword {
        spelling: "setting:",
        means: "setting:SCHEMA/KEY @value=…",
        role: KeywordRole::Prefix,
        build: Some(Statement::Setting),
    },
    Keyword {
        spelling: "exec:",
        means: "exec:./scripts/setup.sh",
        role: KeywordRole::Prefix,
        build: Some(Statement::Exec),
    },
    Keyword {
        spelling: "generate:",
        means: "generate:/path/to/output",
        role: KeywordRole::Prefix,
        build: Some(Statement::Generate),
    },
    Keyword {
        spelling: "dotfiles:",
        means: "dotfiles:./dotfiles @target=~",
        role: KeywordRole::Prefix,
        build: Some(Statement::Dotfiles),
    },
    Keyword {
        spelling: "firewall:",
        means: "firewall:443/tcp",
        role: KeywordRole::Prefix,
        build: Some(Statement::Firewall),
    },
    Keyword {
        spelling: "dir:",
        means: "dir:/path/to/directory",
        role: KeywordRole::Prefix,
        build: Some(Statement::Dir),
    },
    Keyword {
        spelling: "invariant:",
        means: "invariant:NAME @path=…,backed-by=restic:REPO,max-age=7d",
        role: KeywordRole::Prefix,
        build: Some(Statement::Invariant),
    },
    // The directives. No colon, no `build` — each has its own parser above the package
    // parser, but only when written with something after it, so the bare word falls through
    // to the package parser exactly like a resource prefix does.
    Keyword {
        spelling: "use",
        means: "use editors",
        role: KeywordRole::Directive,
        build: None,
    },
    Keyword {
        spelling: "param",
        means: "param gpu = nvidia",
        role: KeywordRole::Directive,
        build: None,
    },
    Keyword {
        spelling: "exclude",
        means: "exclude heavy",
        role: KeywordRole::Directive,
        build: None,
    },
    Keyword {
        spelling: "intersect",
        means: "intersect Work",
        role: KeywordRole::Directive,
        build: None,
    },
    Keyword {
        spelling: "module",
        means: "module editors { … }",
        role: KeywordRole::Directive,
        build: None,
    },
    Keyword {
        spelling: "when",
        means: "when os == linux { … }",
        role: KeywordRole::Directive,
        build: None,
    },
    // Words this grammar does not have, which arrive from the config languages people come
    // from. Each is a real package in a real index (`gem:if`, `npm:else`, `cargo:end`,
    // `gem:import`, `cargo:include`), so left alone they are the one typo that installs
    // software (Q16). `include` additionally has a refusal of its own for `include NAME`.
    Keyword {
        spelling: "if",
        means: "when os == linux { … }",
        role: KeywordRole::Foreign,
        build: None,
    },
    Keyword {
        spelling: "else",
        means: "a second `when` with the opposite condition",
        role: KeywordRole::Foreign,
        build: None,
    },
    Keyword {
        spelling: "end",
        means: "`}` — blocks close with a brace",
        role: KeywordRole::Foreign,
        build: None,
    },
    Keyword {
        spelling: "import",
        means: "use editors",
        role: KeywordRole::Foreign,
        build: None,
    },
    Keyword {
        spelling: "include",
        means: "use editors",
        role: KeywordRole::Foreign,
        build: None,
    },
];

impl Statement {
    /// The `(backend, name)` pair this statement's resource is **listed under**, when there is
    /// a backend by that name.
    ///
    /// `service`, `link` and `setting` are each both a grammar prefix and a registered backend,
    /// and `shall list` prints them in those two columns. So the string a user copies out of a
    /// listing — `service:com.apple.SafariHistoryServiceAgent` — parses as a typed *resource
    /// statement*, not as a `backend:name` package, and every consumer that only understands
    /// packages answered "not installed" about a row `list` had just printed (R-4).
    ///
    /// Read off the variant, never by splitting the line again: which prefix produced this
    /// statement is already known here, and a second place that splits on `:` and trusts what
    /// it finds is the bug `CLAUDE.md` names and `C13` counted six of.
    pub fn listed_as(&self) -> Option<(&'static str, &str)> {
        self.listed_with_options().map(|(pair, _)| pair)
    }

    /// The prefixes [`Statement::listed_as`] can answer with.
    ///
    /// Asking "what kind of thing does this backend hold?" cannot go through a round trip:
    /// `setting:SCHEMA/KEY` is only a legal line once it carries `@value=`, so a name alone
    /// comes back `Nothing` and the guard would call a perfectly writable setting a name no
    /// line can hold — the same false sentence `service:` spent a release printing. Kept
    /// beside `listed_with_options` and pinned to it by a test, because two lists of the same
    /// three prefixes is how one of them silently stops being a resource.
    pub const RESOURCE_BACKENDS: &'static [&'static str] = &["service", "link", "setting"];

    /// As [`Statement::listed_as`], with the options the line carried.
    ///
    /// Rendering a resource line and reading it back has to check both halves: `adopt` writes
    /// a service as the state it found it in, and a line whose *options* came back changed
    /// declares a different state than the one that was observed.
    pub fn listed_with_options(&self) -> Option<((&'static str, &str), &Options)> {
        match self {
            Statement::Service(name, o) => Some((("service", name), o)),
            Statement::Link(name, o) => Some((("link", name), o)),
            Statement::Setting(name, o) => Some((("setting", name), o)),
            // A `firewall:` rule carries an option only when it sets a default policy
            // (`@value=deny`), and the round trip refuses the spelling that does not.
            Statement::Firewall(name, o) => Some((("firewall", name), o)),
            // Every other statement's prefix is not a backend name: `shim:`, `schedule:` and
            // `repo:` name things Shall does rather than things a manager lists, and no
            // registry entry answers to them. Checked, not assumed — `shall list -b shim`
            // refuses as an unknown backend.
            _ => None,
        }
    }
}

mod parse;
mod validate;

pub use parse::parse;
pub use validate::*;
