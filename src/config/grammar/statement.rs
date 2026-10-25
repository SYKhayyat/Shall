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
            | Statement::Dir(_, opts) => Some(opts),
            _ => None,
        }
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
            Statement::Generate(..)
            | Statement::Use(..)
            | Statement::Param { .. }
            | Statement::Exclude(_)
            | Statement::Intersect(_)
            | Statement::Subtract(_)
            | Statement::Expr(_)
            | Statement::Var { .. } => Phase::Resolution,
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
    /// so nothing downstream can be handed one.
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
/// the spec ratchet in `tests/grammar_table_matches_the_spec_tests.rs` above all — would then
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

/// Parse one statement. `line` must already have had comments stripped and be non-blank.
///
/// This is the only `backend:name` parser. Eight existed before, six of which never
/// checked that the prefix named a real backend — so every new prefix (`absent:`, `re:`,
/// `repo:`) was a thing they silently read as a backend name (C13).
pub fn parse(origin: &Origin, line: &str, backends: &dyn BackendNames) -> Result<Statement> {
    let stmt = parse_inner(origin, line, backends)?;
    validate(origin, &stmt)?;
    Ok(stmt)
}

fn parse_inner(origin: &Origin, line: &str, backends: &dyn BackendNames) -> Result<Statement> {
    let line = line.trim();

    if let Some(rest) = line.strip_prefix("use ") {
        return parse_use(origin, rest.trim());
    }
    if line == "use" || line.starts_with("use\t") {
        return parse_use(origin, line[3..].trim());
    }

    // `param NAME` / `param NAME = DEFAULT` (U32). Checked before the package parser so a bare
    // `param gpu` is a parameter declaration, not a package named `param gpu`.
    if let Some(rest) = line.strip_prefix("param ") {
        return parse_param(origin, rest.trim());
    }
    if line == "param" || line.starts_with("param\t") {
        return parse_param(origin, line[5..].trim());
    }

    // II.4's set directives. Checked before the package parser, which would otherwise read
    // `exclude heavy` as a package named `exclude heavy`.
    for word in ["exclude ", "intersect "] {
        if let Some(rest) = line.strip_prefix(word) {
            return parse_set_directive(origin, word.trim(), rest.trim());
        }
    }

    // V.46: `use` already means union, so a second word for it is two ways to do one thing.
    if let Some(rest) = line.strip_prefix("include ") {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`include {}` — there is no `include`", rest.trim()),
        )
        .with_hint(format!(
            "write `use {}`. One word brings something in, everywhere: modules use it too.",
            rest.trim()
        )));
    }

    // A set expression, before the package parser reads `(Work` as a package name — but NOT
    // before the typed statements. `looks_like_expression` fires on `\ | & (`, and a
    // `link:C:\Users\me\.vimrc` is full of `\`: without this guard II.4's set math eats
    // II.2's statements, and `link:` silently parses as `Statement::Expr`. A line that opens
    // with a known statement prefix — or with a known backend, which makes it a package line —
    // is that statement, never an expression.
    if !starts_with_statement_prefix(line)
        && !opens_a_package_line(line, backends)
        && crate::app::profile_expr::looks_like_expression(line)
    {
        return Ok(Statement::Expr(line.to_string()));
    }

    // `-vim`. Checked after expressions so `a \ b` is a difference, not a subtraction.
    if let Some(rest) = line.strip_prefix('-') {
        let target = rest.trim();
        if target.is_empty() {
            return Err(GrammarError::new(origin.clone(), "`-` subtracts nothing")
                .with_hint("write `-vim` to take one package out."));
        }
        reject_leading_dash(origin, target)?;
        return Ok(Statement::Subtract(target.to_string()));
    }

    if let Some(rest) = line.strip_prefix("absent:") {
        let decl = parse_package(origin, rest.trim(), backends)?;
        if decl.backend.is_none() {
            return Err(GrammarError::new(
                origin.clone(),
                format!(
                    "`absent:{}` does not name a backend",
                    decl.selector.as_str()
                ),
            )
            .with_hint(
                "an `absent:` line reaches outside what Shall manages, so it must say which \
                 backend: `absent:apt:libreoffice`.",
            ));
        }
        return Ok(Statement::Absent(decl));
    }

    if let Some(rest) = line.strip_prefix("repo:") {
        let rest = rest.trim();
        // `repo:apt:ppa:deadsnakes/ppa` — backend, then the spec (which has its own colons).
        let Some((backend, spec)) = rest.split_once(':') else {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`repo:{}` does not name a backend", rest),
            )
            .with_hint(
                "a repository belongs to one package manager, so name it: \
                 `repo:apt:ppa:deadsnakes/ppa`. A PPA is apt's, a COPR is dnf's.",
            ));
        };
        let (backend, spec) = (backend.trim(), spec.trim());
        if backend.is_empty() || spec.is_empty() {
            return Err(
                GrammarError::new(origin.clone(), "`repo:` needs `backend:spec`")
                    .with_hint("for example `repo:apt:ppa:deadsnakes/ppa`."),
            );
        }
        if !backends.is_backend(backend) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`{}` is not a backend", backend),
            )
            .with_hint("name the package manager that owns this repository, e.g. `apt`."));
        }
        reject_leading_dash(origin, spec)?;
        return Ok(Statement::Repo {
            backend: backend.to_string(),
            spec: spec.to_string(),
        });
    }

    for kw in KEYWORDS {
        let Some(build) = kw.build else { continue };
        if let Some(rest) = line.strip_prefix(kw.spelling) {
            let (name, options) = split_options(origin, rest.trim())?;
            if name.is_empty() {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`{}` names nothing", kw.spelling),
                ));
            }
            reject_leading_dash(origin, &name)?;
            return Ok(build(name, options));
        }
    }

    if let Some(var) = parse_var(line) {
        return Ok(var);
    }

    // Q16: a bare keyword is not a package name. Checked here, after every form that has a
    // delimiter has been dispatched, so it fires only on the line that would otherwise become
    // a package declaration.
    if let Some(kw) = bare_keyword(line) {
        return Err(keyword_is_not_a_package(origin, kw));
    }

    let decl = parse_package(origin, line, backends)?;
    Ok(Statement::Package(decl))
}

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
            // Every other statement's prefix is not a backend name: `shim:`, `schedule:` and
            // `repo:` name things Shall does rather than things a manager lists, and no
            // registry entry answers to them. Checked, not assumed — `shall list -b shim`
            // refuses as an unknown backend.
            _ => None,
        }
    }
}

/// The keyword a line is nothing but, or `None`.
///
/// A line carrying a `:` is spelled with a prefix — `link:` is the statement and `list:link`
/// / `cargo:link` are the package — so it is never the bare form and is not this function's
/// business. Options are: `link @target=/x` and `link@version=1` are the same typo as `link`
/// with the rest of the line still attached, and mean a package no more than the bare word does.
fn bare_keyword(line: &str) -> Option<&'static Keyword> {
    if line.contains(':') {
        return None;
    }
    let head = line.split([' ', '\t', '@']).next()?;
    KEYWORDS.iter().find(|k| k.word() == head)
}

/// Q16, ruled 2026-07-30. Names both ways to mean the word, because both are reachable and
/// which one was meant is not knowable from the line.
fn keyword_is_not_a_package(origin: &Origin, kw: &Keyword) -> GrammarError {
    let word = kw.word();
    let statement = if kw.takes_colon() {
        format!("to declare that `{}`:", word)
    } else {
        format!("to write the `{}` you meant:", word)
    };
    GrammarError::new(
        origin.clone(),
        format!("`{}` is a keyword, not a package name", word),
    )
    .with_hint(format!(
        "{:<38}{}\n  {:<38}list:{}   (or pin one: cargo:{})",
        statement, kw.means, "to install a package by that name:", word, word,
    ))
}

/// `NAME = VALUE` (IX.2), where NAME is an identifier.
///
/// Checked last, and only for a bare identifier before the `=`, so nothing that is already a
/// package line can be read as a variable: `apt:foo@version=1.2` has a `:` and an `@` in its
/// head, and `-vim` does not start with a name character.
fn parse_var(line: &str) -> Option<Statement> {
    let (head, value) = line.split_once('=')?;
    let name = head.trim();
    if name.is_empty() || !name.starts_with(|c: char| c.is_alphabetic() || c == '_') {
        return None;
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    Some(Statement::Var {
        name: name.to_string(),
        // Verbatim to end of line, trimmed — the same rule as a block-form option value.
        value: value.trim().to_string(),
    })
}

/// Whether a line opens with one of II.2's typed-statement prefixes. Such a line is that
/// statement and must not be mistaken for a set expression (II.4), whatever punctuation its
/// payload carries — a `link:` target is a path, not a difference.
fn starts_with_statement_prefix(line: &str) -> bool {
    KEYWORDS
        .iter()
        .any(|k| k.takes_colon() && line.starts_with(k.spelling))
}

/// Whether a line opens with `<known backend>:`, which makes it a package line (II.2) and not
/// set math (II.4). `\` is a difference operator *and* a legal character in a package name —
/// `winget list` reports 185 such names on a stock Windows box (`ARP\Machine\X64\Firefox`) —
/// so without this the commonest line in the language is read as profile algebra.
///
/// A real difference between two qualified packages (`apt:jq \ apt:vim`) still parses as one:
/// an operator stands apart from its operands and a name never does.
fn opens_a_package_line(line: &str, backends: &dyn BackendNames) -> bool {
    let head = match line.split_whitespace().next() {
        Some(h) => h,
        None => return false,
    };
    let Some((backend, rest)) = head.split_once(':') else {
        return false;
    };
    if rest.is_empty() || !backends.is_backend(backend) {
        return false;
    }
    !has_spaced_operator_outside_quotes(line)
}

/// Whether one of [`SPACED_OPERATORS`] occurs OUTSIDE a quoted span.
///
/// An operator stands apart from its operands — and a name never stands apart from its
/// quotes: `winget:"App Name | Pro"` carries the bytes ` | `, but they belong to the name,
/// and reading them as set algebra made that line unwritable in any module.
fn has_spaced_operator_outside_quotes(line: &str) -> bool {
    let mut in_quotes = false;
    let mut masked = String::with_capacity(line.len());
    for c in line.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
            // The quote itself marks a boundary either way.
            masked.push(' ');
        } else if in_quotes {
            masked.push(' ');
        } else {
            masked.push(c);
        }
    }
    SPACED_OPERATORS.iter().any(|op| masked.contains(op))
}

/// The set operators as they are written between operands. Glued forms (`(a|b)&c`) are still
/// expressions — they cannot open with a backend prefix.
const SPACED_OPERATORS: [&str; 3] = [" \\ ", " | ", " & "];

fn parse_use(origin: &Origin, target: &str) -> Result<Statement> {
    if target.is_empty() {
        return Err(GrammarError::new(origin.clone(), "`use` names nothing")
            .with_hint("write `use editors` (a module) or `use Work` (a profile)."));
    }

    // Split off an optional `(args)` before validating the name, so a `/` inside an argument
    // value (`use m(path=/etc/foo)`) is not mistaken for the path in a `use` target (U32).
    let (name, args) = match target.split_once('(') {
        Some((name, rest)) => {
            let inner = rest.strip_suffix(')').ok_or_else(|| {
                GrammarError::new(
                    origin.clone(),
                    format!("`use {}` opens `(` but never closes it", target),
                )
                .with_hint("write `use workstation(user=shaul, gpu=nvidia)`.")
            })?;
            (name.trim(), parse_use_args(origin, inner)?)
        }
        None => (target, Vec::new()),
    };

    // `use` takes a name, never a path and never a URL (II.2). A file from the internet is
    // a fetch step that puts a module on disk; then you `use` it by name like everything
    // else.
    if name.contains('/') || name.contains('\\') || name.contains("://") {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`use {}` looks like a path or a URL", name),
        )
        .with_hint(
            "`use` takes a name. Fetch the file into `modules/` first, then `use` it by name.",
        ));
    }
    if name.split_whitespace().count() > 1 {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`use {}` names more than one thing", name),
        )
        .with_hint("one `use` per line."));
    }
    let reference = Reference::classify(name).ok_or_else(|| {
        GrammarError::new(
            origin.clone(),
            format!("`{}` is neither a module nor a profile name", name),
        )
        .with_hint("profiles are Capitalized, modules are lowercase.")
    })?;
    // A profile has no parameters to bind (U32): only modules declare `param`.
    if !args.is_empty() && matches!(reference, Reference::Profile(_)) {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`use {}` passes arguments to a profile", name),
        )
        .with_hint("only a module takes parameters (`param`); a profile has none to bind."));
    }
    Ok(Statement::Use(reference, args))
}

/// Parse `k=v, k2=v2` from inside a `use name(...)`. Values are verbatim to the next comma,
/// trimmed. An empty argument list (`use m()`) is allowed and binds nothing.
fn parse_use_args(origin: &Origin, inner: &str) -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    for piece in inner.split(',') {
        let piece = piece.trim();
        if piece.is_empty() {
            continue;
        }
        let Some((k, v)) = piece.split_once('=') else {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`{}` is not a `name=value` argument", piece),
            )
            .with_hint("write each argument as `name=value`, comma-separated."));
        };
        let key = k.trim();
        if key.is_empty() || !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`{}` is not a parameter name", key),
            )
            .with_hint("parameter names are letters, digits and `_`."));
        }
        out.push((key.to_string(), v.trim().to_string()));
    }
    Ok(out)
}

/// `param NAME` / `param NAME = DEFAULT` (U32). The name is an identifier; a default is
/// verbatim to end of line, trimmed, exactly like a `Var` value.
fn parse_param(origin: &Origin, rest: &str) -> Result<Statement> {
    if rest.is_empty() {
        return Err(GrammarError::new(origin.clone(), "`param` names nothing")
            .with_hint("write `param user` (required) or `param gpu = none` (with a default)."));
    }
    let (name, default) = match rest.split_once('=') {
        Some((n, d)) => (n.trim(), Some(d.trim().to_string())),
        None => (rest.trim(), None),
    };
    if name.is_empty()
        || !name.starts_with(|c: char| c.is_alphabetic() || c == '_')
        || !name.chars().all(|c| c.is_alphanumeric() || c == '_')
    {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`{}` is not a parameter name", name),
        )
        .with_hint(
            "parameter names start with a letter or `_` and hold letters, digits and `_`.",
        ));
    }
    Ok(Statement::Param {
        name: name.to_string(),
        default,
    })
}

/// `exclude heavy` / `intersect security` — both take one module or profile name, and case
/// says which, exactly as `use` does.
fn parse_set_directive(origin: &Origin, word: &str, target: &str) -> Result<Statement> {
    if target.is_empty() {
        return Err(
            GrammarError::new(origin.clone(), format!("`{}` names nothing", word)).with_hint(
                format!(
                    "write `{} heavy` (a module) or `{} Work` (a profile).",
                    word, word
                ),
            ),
        );
    }
    if target.split_whitespace().count() > 1 {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`{} {}` names more than one thing", word, target),
        )
        .with_hint(format!("one `{}` per line.", word)));
    }
    let reference = Reference::classify(target).ok_or_else(|| {
        GrammarError::new(
            origin.clone(),
            format!("`{}` is neither a module nor a profile name", target),
        )
        .with_hint("profiles are Capitalized, modules are lowercase.")
    })?;
    Ok(match word {
        "exclude" => Statement::Exclude(reference),
        _ => Statement::Intersect(reference),
    })
}

/// Split `NAME@opts` into its name and options. Used by the non-package statements, whose
/// names are not `backend:name` (`shim:jq@source=cargo:jq`).
fn split_options(origin: &Origin, text: &str) -> Result<(String, Options)> {
    match text.split_once('@') {
        Some((name, opts)) => Ok((name.trim().to_string(), parse_short(origin, opts)?)),
        None => Ok((text.to_string(), Options::default())),
    }
}

/// Read the `backend:` prefix of a package line, which may name a chain.
///
/// `apt` pins. `apt,dnf` and `apt,list` do not — they say what the line will accept, in
/// order, and leave the choosing to the machine. A comma rather than a hyphen because
/// package managers have hyphens in their names (`nix-env`, `apt-get`), and a separator a
/// backend name can contain is a separator that stops working the day such a backend is
/// added.
fn parse_prefix(
    origin: &Origin,
    prefix: &str,
    backends: &dyn BackendNames,
) -> Result<(Option<String>, Candidates)> {
    // A group is a shortcut for a comma-chain (U18), so expand it into that chain BEFORE any
    // validation — every member then goes through the same backend check and `list`-only rules a
    // hand-written chain does. `tools:rg` becomes `apt,dnf,cargo:rg`, and `tools,brew:rg` splices
    // the group's members in front of `brew`. Nested groups are already flattened to terminal
    // backends by `Groups` (a cycle was refused at load), so a single expansion here is complete.
    let mut expanded: Vec<String> = Vec::new();
    for raw in prefix.split(',').map(str::trim) {
        match backends.expand_group(raw) {
            Some(members) => expanded.extend(members),
            None => expanded.push(raw.to_string()),
        }
    }
    let parts: Vec<&str> = expanded.iter().map(String::as_str).collect();

    let unknown = |name: &str| {
        GrammarError::new(
            origin.clone(),
            format!("`{}` is not a backend Shall uses", name),
        )
        .with_hint(format!(
            "add `{}` to your `priority` file, or check the spelling. Not listed means \
             Shall does not use it at all.",
            name
        ))
    };

    // `list` is only a tail: everything after it would never be reached, and writing
    // something unreachable means the line does not say what its author thinks it says.
    if let Some(pos) = parts.iter().position(|p| *p == PRIORITY_KEYWORD) {
        if pos != parts.len() - 1 {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`{}` must come last in `{}`", PRIORITY_KEYWORD, prefix),
            )
            .with_hint(format!(
                "`{}` already means every manager in `priority`, so nothing written after \
                 it can ever be reached.",
                PRIORITY_KEYWORD
            )));
        }
    }

    let mut named: Vec<String> = Vec::new();
    let mut ends_in_priority = false;
    for part in &parts {
        if part.is_empty() {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`{}` has an empty backend in it", prefix),
            )
            .with_hint("write `apt,dnf:rg` — one manager between each comma."));
        }
        if *part == PRIORITY_KEYWORD {
            ends_in_priority = true;
            continue;
        }
        if !backends.is_backend(part) {
            return Err(unknown(part));
        }
        if named.iter().any(|n| n == part) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`{}` is named twice in `{}`", part, prefix),
            )
            .with_hint("the first one already decides; the second can never be reached."));
        }
        named.push(part.to_string());
    }

    Ok(match (named.len(), ends_in_priority) {
        // `list:rg`, which is what a bare `rg` means spelled out.
        (0, true) => (None, Candidates::Priority),
        (0, false) => return Err(unknown(prefix)),
        // One manager and no tail is the pin: apt or nothing.
        (1, false) => (Some(named.remove(0)), Candidates::Priority),
        (_, false) => (None, Candidates::Named(named)),
        (_, true) => (None, Candidates::NamedThenPriority(named)),
    })
}

/// A name reaches a manager's command line, where a leading `-` is an option and not a name.
/// The `--` every invocation emits (II.12b) holds for managers that honour it; this holds for
/// the rest, and it is the layer that can say *which line* is wrong.
fn reject_leading_dash(origin: &Origin, name: &str) -> Result<()> {
    if name.starts_with('-') {
        return Err(GrammarError::new(
            origin.clone(),
            format!(
                "`{}` starts with `-`, so it is an option and not a package name",
                name
            ),
        )
        .with_hint(
            "package names reach the manager's command line. If you meant to take a package \
             out of the set, a subtraction is `-name` at the start of its own line.",
        ));
    }
    Ok(())
}

/// Where a package line's options begin, if they do.
///
/// An `@` that **opens the name** is part of the name (Q23). npm's scoped packages are named
/// `@scope/name` — `@angular/cli`, `@bazel/bazelisk` — and `npm ls -g` prints them, so a name
/// Shall lists has to be a name Shall can be given back. Before this, `npm:@bazel/bazelisk` was
/// read as an empty name followed by an option list and refused with *"is not a list of
/// `key=value` options"*, which is baffling advice about a line nobody wrote wrongly.
///
/// Only the first character of the name is special. Every later `@` still opens the options,
/// which is what keeps `npm:@scope/name@version=1.2` a pinned scoped package rather than a
/// package called `@scope/name@version=1.2`.
fn option_separator(text: &str) -> Option<usize> {
    // Where the name starts: after the backend prefix and any space that follows it.
    let after_prefix = text.find(':').map(|i| i + 1).unwrap_or(0);
    let name_start = text[after_prefix..]
        .find(|c: char| !c.is_whitespace())
        .map(|off| after_prefix + off)
        .unwrap_or(after_prefix);
    // A quoted name is opaque: everything up to the closing quote is the name, `@` included.
    // Without this, `winget:"Some App@2"` would split inside the quotes and leave an unbalanced
    // name and an option list made of the rest of somebody's package.
    match quoted_span(text, name_start) {
        // The options begin *at* the character after the closing quote, so the `@` that opens
        // them sits exactly on that index — the one place a strict `>` silently loses it.
        Some(end) => text[end..].find('@').map(|off| end + off),
        None => text
            .char_indices()
            .find(|(i, c)| *c == '@' && *i != name_start)
            .map(|(i, _)| i),
    }
}

/// The index just past the closing quote, when `at` opens a quoted name.
///
/// `None` when there is no quote there or the quote is never closed — an unterminated quote is
/// left to the name parser, which has the origin needed to say so in a way the user can act on.
fn quoted_span(text: &str, at: usize) -> Option<usize> {
    let rest = text.get(at..)?;
    if !rest.starts_with('"') {
        return None;
    }
    rest[1..].find('"').map(|close| at + close + 2)
}

fn parse_package(origin: &Origin, text: &str, backends: &dyn BackendNames) -> Result<PackageDecl> {
    let (head, options) = match option_separator(text) {
        Some(at) => (text[..at].trim(), parse_short(origin, &text[at + 1..])?),
        None => (text, Options::default()),
    };

    if head.is_empty() {
        return Err(GrammarError::new(origin.clone(), "no package name"));
    }

    // Checked before the backend split so `re:^fonts-` gets the error that says what is
    // missing, rather than "`re` is not a backend" — which is true but useless.
    if let Some(pattern) = head.strip_prefix("re:") {
        return Err(GrammarError::new(
            origin.clone(),
            format!(
                "`re:{}` does not say which backend to match in",
                pattern.trim()
            ),
        )
        .with_hint("write `apt:re:^fonts-`. A pattern has to be matched somewhere."));
    }

    let (backend, candidates, rest) = match head.split_once(':') {
        Some((prefix, rest)) => {
            let (backend, candidates) = parse_prefix(origin, prefix, backends)?;
            (backend, candidates, rest.trim())
        }
        None => (None, Candidates::Priority, head),
    };

    let selector = match rest.strip_prefix("re:") {
        Some(pattern) => {
            let pattern = pattern.trim();
            if pattern.is_empty() {
                return Err(GrammarError::new(origin.clone(), "`re:` has no pattern"));
            }
            // A pattern is matched against one manager's catalogue and frozen in that
            // manager's regex lock. Spread over a chain there is no single catalogue to
            // match and no single lock to write, so the line has to pin.
            if backend.is_none() {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`{}` must match in exactly one backend", head),
                )
                .with_hint(
                    "write `apt:re:^fonts-`. A pattern is matched against one manager's \
                     catalogue, so a chain has nothing to match against.",
                ));
            }
            Selector::Regex(pattern.to_string())
        }
        None => {
            if rest.is_empty() {
                return Err(GrammarError::new(
                    origin.clone(),
                    "no package name after the backend",
                ));
            }
            // A quoted name is taken verbatim, spaces and all. `winget list` answers with
            // `ARP\Machine\X64\Mozilla Firefox` — the identifier `winget install` takes back —
            // and a name Shall lists has to be a name Shall can be given (V.113). Quoting is
            // what keeps that from re-opening VI.1: prose is not quoted, so a typo is still an
            // error rather than a package named after itself.
            let name = match rest.strip_prefix('"') {
                Some(inner) => match inner.strip_suffix('"') {
                    Some(name) => {
                        if name.trim().is_empty() {
                            return Err(GrammarError::new(origin.clone(), "`\"\"` names nothing"));
                        }
                        // The quotes carry exactly what is between them, so an edge nobody can
                        // see is a second package that reads as the first: `"Firefox "` and
                        // `Firefox` are one thing declared twice, and every message about
                        // either of them prints the same word.
                        if name.trim() != name {
                            return Err(GrammarError::new(
                                origin.clone(),
                                format!("`{}` has a space at the edge of the name", rest),
                            )
                            .with_hint(format!(
                                "a quoted name is exact, so this is not the same package as \
                                 `\"{}\"` — which is almost certainly the one you mean.",
                                name.trim()
                            )));
                        }
                        if name.contains('"') {
                            return Err(GrammarError::new(
                                origin.clone(),
                                format!("`{}` has a quote inside a quoted name", rest),
                            )
                            .with_hint(
                                "a quoted name runs to the next quote; there is no escape for \
                                 one inside it.",
                            ));
                        }
                        // A manifest is a line per declaration. A name carrying a newline
                        // round-trips through this parser and then writes two lines — which is
                        // the wedged-config bug quoting exists to end, re-entering by the door
                        // quoting opened.
                        if name.chars().any(char::is_control) {
                            return Err(GrammarError::new(
                                origin.clone(),
                                "a quoted name contains a control character".to_string(),
                            )
                            .with_hint(
                                "a declaration is one line; a name that carries a newline or a \
                                 tab cannot be written as one.",
                            ));
                        }
                        name
                    }
                    // Distinguished, because "you never closed the quote" is unhelpful advice
                    // about a line that closed it and then kept going.
                    None => {
                        return Err(match inner.find('"') {
                            Some(close) => GrammarError::new(
                                origin.clone(),
                                format!("`{}` has more after the closing quote", rest),
                            )
                            .with_hint(format!(
                                "the name ends at the quote — `{}`. Options go after it, as \
                                 `@key=value`.",
                                &inner[..close]
                            )),
                            None => GrammarError::new(
                                origin.clone(),
                                format!("`{}` opens a quote and never closes it", rest),
                            )
                            .with_hint("write `winget:\"ARP\\Machine\\X64\\Mozilla Firefox\"`."),
                        });
                    }
                },
                // A package name is one word. Without this, any unrecognised prose becomes a
                // package literally named after itself — VI.1's "any typo becomes a package
                // name", which is what II.2's "an unrecognised line is an error" forbids.
                None => {
                    if rest.split_whitespace().count() > 1 {
                        return Err(GrammarError::new(
                            origin.clone(),
                            format!("`{}` is not a package name", rest),
                        )
                        .with_hint(
                            "a name with spaces in it has to be quoted: \
                             `winget:\"Mozilla Firefox\"`.",
                        ));
                    }
                    rest
                }
            };
            reject_leading_dash(origin, name)?;
            Selector::Name(name.to_string())
        }
    };

    Ok(PackageDecl {
        backend,
        candidates,
        selector,
        options,
    })
}

/// Every option rule in II.2, for every statement that carries options.
///
/// This runs on the finished statement rather than inside the header parse, because a block
/// body's keys are merged in after the header is parsed. Validating at the header let
/// `apt:jq@hold { version = 1.6 }` through — the same contradiction the short form refuses,
/// silent — and II.2 closes with the reason that cannot stand: silently ignoring an option
/// the user wrote is how a config grows lines that do nothing.
pub fn validate(origin: &Origin, stmt: &Statement) -> Result<()> {
    match stmt {
        Statement::Package(decl) => validate_options(origin, decl, false),
        Statement::Absent(decl) => validate_options(origin, decl, true),
        Statement::Shim(name, o) => validate_extra_options(origin, OptionKind::Shim, name, o, None),
        Statement::Service(name, o) => {
            validate_extra_options(origin, OptionKind::Service, name, o, None)
        }
        Statement::Link(name, o) => validate_link(origin, name, o),
        Statement::Schedule(name, o) => {
            validate_extra_options(origin, OptionKind::Schedule, name, o, None)
        }
        Statement::Setting(name, o) => validate_setting(origin, name, o),
        Statement::Exec(name, o) => super::exec::validate_exec(origin, name, o),
        Statement::Generate(name, o) => super::exec::validate_generate(origin, name, o),
        Statement::Dotfiles(name, o) => {
            validate_extra_options(origin, OptionKind::Dotfiles, name, o, None)
        }
        Statement::Firewall(name, o) => validate_firewall(origin, name, o),
        Statement::Dir(name, o) => validate_extra_options(origin, OptionKind::Dir, name, o, None),
        Statement::Repo { .. }
        | Statement::Use(..)
        | Statement::Param { .. }
        | Statement::Exclude(_)
        | Statement::Intersect(_)
        | Statement::Subtract(_)
        | Statement::Var { .. }
        | Statement::Expr(_) => Ok(()),
    }
}

/// The options each non-package statement understands (II.2's table).
///
/// A `schedule:` also needs `cron` and `run` to be *present*, which `model::schedule` checks
/// when it builds the job — that is a question about one line's meaning, not about which
/// words are legal, and it has an error that can name what is missing.
/// `scope` is on exactly the three statements where "for me" and "for the machine" can differ
/// (U19). A `service:` is the init system's business and a `repo:` is the manager's, so
/// neither takes it — a key that means nothing on a statement is a key that will be written
/// there and silently ignored.
/// A statement kind that carries `@options`, as the option tables are keyed.
///
/// An enum and not the `&str` from [`Statement::kind`]: the lookup below used to match on the
/// string with `_ => SCHEDULE_OPTION_KEYS` as its fall-through, so a kind added without a table
/// silently inherited schedule's — `@cron` accepted on something that has no schedule, its own
/// options refused, and no complaint from anywhere. The compiler asks the question now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionKind {
    Shim,
    Service,
    Link,
    Schedule,
    Setting,
    Exec,
    Generate,
    Dotfiles,
    Firewall,
    Dir,
}

impl OptionKind {
    /// The keyword as written, for the refusal that names it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shim => "shim",
            Self::Service => "service",
            Self::Link => "link",
            Self::Schedule => "schedule",
            Self::Setting => "setting",
            Self::Exec => "exec",
            Self::Generate => "generate",
            Self::Dotfiles => "dotfiles",
            Self::Firewall => "firewall",
            Self::Dir => "dir",
        }
    }

    /// Every kind that carries options — what a test quantifies over, so a kind added without
    /// a case in one of them is a failure and not an omission.
    pub const ALL: &'static [Self] = &[
        Self::Shim,
        Self::Service,
        Self::Link,
        Self::Schedule,
        Self::Setting,
        Self::Exec,
        Self::Generate,
        Self::Dotfiles,
        Self::Firewall,
        Self::Dir,
    ];
}

pub const SHIM_OPTION_KEYS: &[&str] = &["source", "scope", "user"];
pub const SERVICE_OPTION_KEYS: &[&str] = &["enabled", "status", "user"];
pub const LINK_OPTION_KEYS: &[&str] = &[
    "target", "content", "template", "decrypt", "identity", "scope", "backup", "user", "owner",
];
/// `enabled`, `persistent`, `jitter` and `elevated` are here even though no scheduler expresses
/// all four: the grammar says what a schedule may *say*, and `app/scheduler` says what each OS
/// can *keep*. Refusing an option in the parser because one platform cannot hold it would make
/// a portable model file unreadable on the machine it was written for.
pub const SCHEDULE_OPTION_KEYS: &[&str] = &[
    "cron",
    "run",
    "notify",
    "enabled",
    "persistent",
    "jitter",
    "elevated",
    "user",
];
pub const SETTING_OPTION_KEYS: &[&str] = &["value", "scope", "user"];
/// `target` is where the tree is mirrored to; absent means the home directory, which is what a
/// dotfiles tree mirrors by definition. There is deliberately no per-file option: the tree has
/// no place to write one, which is why it never decrypts (U24).
pub const DOTFILES_OPTION_KEYS: &[&str] = &["target"];
/// `value` is the policy a `default/...` rule sets (`allow` or `deny`). A port rule takes no
/// options: `firewall:22/tcp` is the whole declaration.
pub const FIREWALL_OPTION_KEYS: &[&str] = &["value"];
pub const DIR_OPTION_KEYS: &[&str] = &["user", "owner", "mode"];

pub const USER_SCOPABLE_KINDS: &[ResourceKind] = &[
    ResourceKind::Link,
    ResourceKind::Dir,
    ResourceKind::Shim,
    ResourceKind::Setting,
    ResourceKind::Service,
    ResourceKind::Schedule,
];

pub fn set_user_scope(origin: &Origin, stmt: &mut Statement, user: &str) -> Result<()> {
    let kind = stmt.kind();
    if !kind.is_some_and(|kind| USER_SCOPABLE_KINDS.contains(&kind)) {
        return Err(GrammarError::new(
            origin.clone(),
            format!(
                "`use user:NAME {{ ... }}` does not support `{}` statements",
                stmt.kind().map_or("this", ResourceKind::as_str)
            ),
        ));
    }
    let options = match stmt {
        Statement::Link(_, options)
        | Statement::Dir(_, options)
        | Statement::Shim(_, options)
        | Statement::Setting(_, options)
        | Statement::Service(_, options)
        | Statement::Schedule(_, options) => options,
        _ => {
            return Err(GrammarError::new(
                origin.clone(),
                format!(
                    "`use user:NAME {{ ... }}` does not support `{}` statements",
                    stmt.kind().map_or("this", ResourceKind::as_str)
                ),
            ))
        }
    };
    if !options.contains("user") {
        options.set("user", user);
    }
    Ok(())
}

/// The option keys that answer to ONE value, across every kind.
///
/// A key given twice is a list (II.2) for the keys that mean it — `@requires=`, `@after=` —
/// and a silent first-wins demotion for every other. This is the set of the others: a repeat
/// is refused at [`validate_extra_options`] and in the package path below instead of being
/// demoted by `one()`'s `.first()`. Deliberately absent: `requires`, `after` (lists by
/// design), `on` and `notify` (comma-bearing values whose multiplicity lives in the value).
pub(crate) const SINGLE_VALUE_OPTION_KEYS: &[&str] = &[
    "source",
    "enabled",
    "status",
    "target",
    "content",
    "template",
    "decrypt",
    "identity",
    "backup",
    "cron",
    "undo",
    "value",
    "scope",
    "user",
    "owner",
    "mode",
    "version",
    "hold",
    "expires",
    "until",
    "sha256",
    "bin",
    "channel",
    "asset",
    "download_only",
    "runs",
];

/// A `link:` names one mode — inline `@content=`, a `@template=true` source file, a
/// `@decrypt=` source, or a plain symlink — and the installer reads them in that order.
/// A line naming two is a line whose second mode nobody reads: `content` wins in the
/// installer while `check` used to call the same line unverifiable, so the refusal lives
/// here, where the line is written, rather than in either reader. `@template=` is a flag
/// with a value, so anything but `true` or `false` is refused rather than read as a plain
/// symlink that happens to carry a word nobody acts on.
fn validate_link(origin: &Origin, name: &str, options: &Options) -> Result<()> {
    validate_extra_options(origin, OptionKind::Link, name, options, None)?;
    if let Some(t) = options.one("template") {
        if t != "true" && t != "false" {
            return Err(GrammarError::new(
                origin.clone(),
                format!(
                    "`link:{}` has `@template={}`, which is neither on nor off",
                    name, t
                ),
            )
            .with_hint(
                "write `@template` or `@template=true` to render, or drop it for a plain link.",
            ));
        }
    }
    // `@template=false` states the default and is not a mode, so it never collides:
    // only a `true` template competes with content and decrypt.
    let mut modes = Vec::new();
    if options.contains("content") {
        modes.push("content");
    }
    if options.contains("decrypt") {
        modes.push("decrypt");
    }
    if options.one("template") == Some("true") {
        modes.push("template");
    }
    if modes.len() > 1 {
        return Err(GrammarError::new(
            origin.clone(),
            format!(
                "`link:{}` names two modes (`@{}` and `@{}`), and only the first one runs",
                name, modes[0], modes[1]
            ),
        )
        .with_hint("a `link:` is inline `@content=`, a `@template=true` source, a `@decrypt=` source, or a plain link — pick one."));
    }
    Ok(())
}

/// A firewall line names a rule the grammar can read, and a default policy says which one.
fn validate_firewall(origin: &Origin, name: &str, options: &Options) -> Result<()> {
    validate_extra_options(origin, OptionKind::Firewall, name, options, None)?;
    let rule = crate::model::firewall::Rule::parse(name)
        .map_err(|e| GrammarError::new(origin.clone(), e))?;
    match rule {
        crate::model::firewall::Rule::Default { .. } => match options.one("value").map(str::trim) {
            Some("allow") | Some("deny") => Ok(()),
            _ => Err(GrammarError::new(
                origin.clone(),
                format!("`firewall:{}` needs a policy", name),
            )
            .with_hint(
                "say which way it goes: `@value=deny` or `@value=allow`. A default policy \
                     with no value declares nothing, and it is the most consequential line in \
                     a firewall.",
            )),
        },
        // A port rule is its own declaration; `@value=` on one would be a second way to say
        // the same thing, and a confusing one (`firewall:22/tcp @value=deny` reads as both).
        crate::model::firewall::Rule::Port { .. } => match options.one("value") {
            None => Ok(()),
            Some(_) => Err(GrammarError::new(
                origin.clone(),
                format!("`firewall:{}` takes no `value`", name),
            )
            .with_hint(
                "a declared port is open — that is what declaring it means. To close one, \
                 delete the line; `@value=` belongs on `default/incoming` only.",
            )),
        },
    }
}

/// [`keys_for`] for a test in another binary: the table a kind reads, so a kind wired to the
/// wrong one — which still compiles — can be caught by quantifying over [`OptionKind::ALL`].
pub fn keys_for_kind(kind: OptionKind) -> &'static [&'static str] {
    keys_for(kind)
}

/// The options one kind may carry. **Exhaustive, with no default arm** — a tenth kind does not
/// compile until it says which options it takes.
fn keys_for(kind: OptionKind) -> &'static [&'static str] {
    match kind {
        OptionKind::Shim => SHIM_OPTION_KEYS,
        OptionKind::Service => SERVICE_OPTION_KEYS,
        OptionKind::Link => LINK_OPTION_KEYS,
        OptionKind::Schedule => SCHEDULE_OPTION_KEYS,
        OptionKind::Setting => SETTING_OPTION_KEYS,
        OptionKind::Exec => super::exec::EXEC_OPTION_KEYS,
        OptionKind::Generate => super::exec::GENERATE_OPTION_KEYS,
        OptionKind::Dotfiles => DOTFILES_OPTION_KEYS,
        OptionKind::Firewall => FIREWALL_OPTION_KEYS,
        OptionKind::Dir => DIR_OPTION_KEYS,
    }
}

/// An `exec:` names a script and, optionally, how many times its content may run. The name
/// must be non-empty; `runs`, if present, is a positive count or the word `always`.
/// `@scope=` must name one of the two things it can mean. A misspelling that parsed as
/// "default" would be a line that reads as a decision and behaves as if nobody made one.
fn validate_scope(origin: &Origin, prefix: &str, name: &str, options: &Options) -> Result<()> {
    let Some(written) = options.one("scope") else {
        return Ok(());
    };
    if crate::model::scope::Scope::parse(written).is_none() {
        return Err(GrammarError::new(
            origin.clone(),
            format!("`{}:{}` has an invalid `scope={}`", prefix, name, written),
        )
        .with_hint(format!(
            "scope is {}. Omitting it means whatever this store does by default.",
            crate::model::scope::Scope::vocabulary()
        )));
    }
    Ok(())
}

/// Split `SCHEMA/KEY` into its halves. The one place the shape is decided, so the parser's
/// refusal and the adapter's lookup cannot disagree about what a setting names.
pub fn split_setting(name: &str) -> Option<(&str, &str)> {
    let (schema, key) = name.split_once('/')?;
    let (schema, key) = (schema.trim(), key.trim());
    if schema.is_empty() || key.is_empty() || key.contains('/') {
        return None;
    }
    Some((schema, key))
}

/// A setting names a schema, a key inside it, and the value it must hold. A line missing any
/// of the three describes no state, and applying it would mean choosing on the user's behalf
/// which key they meant.
fn validate_setting(origin: &Origin, name: &str, options: &Options) -> Result<()> {
    validate_extra_options(origin, OptionKind::Setting, name, options, None)?;

    if split_setting(name).is_none() {
        return Err(
            GrammarError::new(origin.clone(), format!("`{}` is not `SCHEMA/KEY`", name)).with_hint(
                "a setting names the schema and the key inside it, separated by one `/`: \
             `setting:org.gnome.desktop.interface/color-scheme @value=prefer-dark`.",
            ),
        );
    }

    if options.one("value").is_none_or(str::is_empty) {
        return Err(
            GrammarError::new(origin.clone(), format!("`setting:{}` has no value", name))
                .with_hint(
                    "say what the key must hold: `@value=prefer-dark`. A setting with no value \
                     declares nothing.",
                ),
        );
    }
    // A repeated `@value=` is refused one gate up, in `validate_extra_options`, with every
    // other single-valued key.
    Ok(())
}

/// Refuse any option this kind does not take, then check `@scope=` if it took one.
///
/// `hint` is the kind's own sentence about what its options *mean*. The generic
/// "takes: a, b, c" lists the keys and explains none of them, and a refusal that only names
/// the legal spellings makes the reader go and look up what they do — so each caller keeps the
/// sentence it had, and only the *table* is shared.
pub(super) fn validate_extra_options(
    origin: &Origin,
    kind: OptionKind,
    name: &str,
    options: &Options,
    hint: Option<&str>,
) -> Result<()> {
    let prefix = kind.as_str();
    let legal = keys_for(kind);
    for key in options.keys() {
        if legal.contains(&key) {
            // A key that answers to ONE value refuses a second occurrence outright. `one()`
            // reads the first, so a repeated `@status=running,@status=stopped` kept the first
            // and did nothing — a hand-edit flip that parsed, exited 0, changed nothing.
            // Cardinality used to be checked for five keys one `if` at a time; this is the
            // same refusal for every single-valued key in the grammar, at the one gate every
            // typed line passes.
            if SINGLE_VALUE_OPTION_KEYS.contains(&key) && options.all(key).len() > 1 {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`@{key}` takes one value, given {}", options.all(key).len()),
                )
                .with_hint(
                    "`one()` of a repeated key reads the FIRST value, so the second was \
                     silently discarded. Write the one you mean.",
                ));
            }
            continue;
        }
        let what = if legal.is_empty() {
            format!(
                "`{}:{}` has an option `{}`, but it takes none",
                prefix, name, key
            )
        } else {
            format!("`@{}` is not an option on `{}:`", key, prefix)
        };
        let hint = match hint {
            Some(h) => h.to_string(),
            None => format!("`{}:{}` takes: {}.", prefix, name, legal.join(", ")),
        };
        return Err(GrammarError::new(origin.clone(), what).with_hint(hint));
    }
    validate_scope(origin, prefix, name, options)
}

/// Every option a package line may carry (II.2's table). Hooks are `*_install`
/// (`after_install`, `before_install`, …), so they are matched by suffix rather than listed.
///
/// `until` is here and refused below unless the line is `absent:` — II.2 puts it on
/// `absent:` only, and "not an option" would be the wrong error for a key that exists.
pub(crate) const PACKAGE_OPTION_KEYS: &[&str] = &[
    "version",
    "hold",
    "expires",
    "until",
    "requires",
    "sha256",
    "formats",
    "asset",
    "bin",
    "channel",
    "allow_http",
    "unverified",
    // Q49. "Write into the environment the OS owns" — legal only where a manager can be told
    // that, and refused by name everywhere else. `capability::OS_OWNED_ENV` is the one table.
    "system",
    "health",
    "download_only",
    // U39. Legal only on a backend that installs from something other than the name, and
    // refused by name everywhere else — `capability::INSTALLS_FROM_SOURCE` is the one table.
    "url",
    // A shim is a PATH stand-in that forwards to a managed tool. `@shim=true` asks for one on
    // the tool's own line — the form R3 named when it deleted the imperative command — and
    // `@sandbox=true` asks for the same shim and confines `shall run` as well. Both are read by
    // `sync`, and this table refused them until Q18, so the form the ruling pointed at was the
    // one form that did not parse.
    "shim",
    "sandbox",
    // Q18. The geometry of a declared storage object, and snap's confinement. Legal on the
    // backends that read them and refused by name elsewhere — `capability::SCOPED_OPTIONS` is
    // the one table, and the test beside it asserts every key there appears here.
    "size",
    "quota",
    "mount",
    "mount_options",
    "classic",
    // Q19. A declared `@size` is applied to a volume that already exists, and the one direction
    // that can destroy data says so on the line.
    "allow_shrink",
];

pub fn package_option_keys() -> &'static [&'static str] {
    PACKAGE_OPTION_KEYS
}

/// Whether `key` names an option a package line may carry.
///
/// The one predicate over II.2's table. It exists because the lexer needs the same answer the
/// validator needs: `@version=1.0.0@hold` has to be told apart from `@source=owner/repo@v2`, and
/// the only thing that distinguishes them is whether the text after the `@` names an option.
pub(crate) fn is_package_option_key(key: &str) -> bool {
    PACKAGE_OPTION_KEYS.contains(&key) || key.ends_with("_install")
}

/// Options that are only meaningful on some backends — one that resolves a name to several
/// downloadable artifacts, one that publishes version streams, one that installs from a URL,
/// one that carves up a disk. Each is refused by name on any other backend: an option nobody
/// reads is a line that does nothing.
/// Takes the backend and the options rather than a declaration, because the same rules apply
/// to a backend's options body in `priority` (VIII.2) and one of them had to be the caller.
pub fn validate_backend_options(origin: &Origin, backend: Option<&str>, o: &Options) -> Result<()> {
    use crate::backends::artifact::{AssetPattern, FormatOrder};
    use crate::backends::capability;

    for key in ["formats", "asset", "bin"] {
        if !o.contains(key) {
            continue;
        }
        // A line with no prefix is resolved through `priority` later, so the backend that will
        // answer it is not known here. Refusing would break `fd@formats=deb`; the resolver
        // enforces it once the backend is known.
        let Some(backend) = backend else { continue };
        if !capability::selects_artifacts(backend) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`@{}` is not an option on `{}`", key, backend),
            )
            .with_hint(format!(
                "`{}` picks between several files of one release. Backends that offer a \
                 choice: {}. Everywhere else the ecosystem already decided the file.",
                key,
                capability::artifact_backends()
            )));
        }
    }

    if o.contains("channel") {
        if let Some(backend) = backend {
            if !capability::has_channels(backend) {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`@channel` is not an option on `{}`", backend),
                )
                .with_hint(format!(
                    "a channel is a version stream, not a file. Backends that publish \
                     channels: {}.",
                    capability::channel_backends()
                )));
            }
        }
        if o.all("channel").len() > 1 {
            return Err(
                GrammarError::new(origin.clone(), "`@channel` takes one value").with_hint(
                    "there is no fallback across version streams — trying `edge` and settling for \
                 `stable` would silently downgrade the machine. Name the one you want.",
                ),
            );
        }
    }

    for name in o.all("formats") {
        FormatOrder::parse_all([name])
            .map_err(|e| GrammarError::new(origin.clone(), e.to_string()))?;
    }

    if let Some(pattern) = o.one("asset") {
        AssetPattern::parse(pattern)
            .map_err(|e| GrammarError::new(origin.clone(), e.to_string()))?;
    }

    // `@download_only` (D3b) means "fetch but do not install" — a distinction only a backend
    // that downloads a file can draw. Every other backend hands the whole job to a package
    // manager, so there is no fetch-without-install to ask for.
    if o.contains("download_only") {
        if let Some(backend) = backend {
            if !capability::downloads(backend) {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`@download_only` is not an option on `{}`", backend),
                )
                .with_hint(format!(
                    "it fetches a file without installing it, which only {} do.",
                    capability::download_backends()
                )));
            }
        }
    }

    // U39's install source. On a backend that installs by name it is a line that does nothing,
    // and — worse than the artifact options above — it would read as the *name* being wrong.
    for key in PACKAGE_OPTION_KEYS
        .iter()
        .filter(|k| capability::is_source_key(k))
    {
        if !o.contains(key) {
            continue;
        }
        let Some(backend) = backend else { continue };
        if capability::install_source_key(backend).is_none() {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`@{}` is not an option on `{}`", key, backend),
            )
            .with_hint(format!(
                "it says where to install a package from, for a manager that installs by one \
                 string and removes by another: {}. Everywhere else the name is both.",
                capability::source_backends(key)
            )));
        }
    }

    // Q18's family: a key one kind of backend reads and no other can act on. Same shape as the
    // install source above and for the same reason — on `apt` a `@quota=` would be read as the
    // machine having been told something, when nothing anywhere would act on it.
    for key in PACKAGE_OPTION_KEYS
        .iter()
        .filter(|k| capability::is_scoped_option(k))
    {
        if !o.contains(key) {
            continue;
        }
        let Some(backend) = backend else { continue };
        if !capability::takes_scoped_option(backend, key) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`@{}` is not an option on `{}`", key, backend),
            )
            .with_hint(capability::scoped_option_reason(key)));
        }
    }

    // `@mount_options` fills the option field of the fstab entry `@mount` writes. Without
    // `@mount` there is no entry, so the key is read by nothing — the same "line that does
    // nothing" the refusals above exist to prevent, one level in.
    if o.contains("mount_options") && !o.contains("mount") {
        return Err(GrammarError::new(
            origin.clone(),
            "`@mount_options` has no `@mount` to apply to",
        )
        .with_hint(
            "it fills the option field of the fstab entry `@mount=` writes, so on its own it \
             reaches nothing: add `@mount=/where`, or drop it.",
        ));
    }

    // Q19's opt-out, and the same one-level-in rule. `@allow_shrink` permits a *declared* size to
    // take space back, so without `@size` there is no declaration for it to permit — and a line
    // that reads as "shrinking is allowed here" while nothing can shrink is worse than a line
    // that does nothing, because someone will believe it.
    if o.contains("allow_shrink") && !o.contains("size") {
        return Err(GrammarError::new(
            origin.clone(),
            "`@allow_shrink` has no `@size` to apply to",
        )
        .with_hint(
            "it lets a smaller `@size=` take space back off an existing volume, so on its own \
             it permits nothing: add `@size=`, or drop it.",
        ));
    }

    // SEC2's two opt-outs each relax a rule, and the rules have different reach — which is why
    // they are checked apart and never as a pair. Plain HTTP is only a question where Shall
    // itself fetches a URL; a checksum is a question wherever *something* vouches for the
    // bytes, and a manager can be that something (Q5). On a backend that verifies nothing
    // either flag is a line that does nothing, which II.2 refuses.
    if o.contains("allow_http") {
        if let Some(backend) = backend.filter(|b| !capability::downloads(b)) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`@allow_http` is not an option on `{}`", backend),
            )
            .with_hint(format!(
                "it relaxes a rule about downloading and running a file, which only {} do. \
                 Everywhere else the package manager chose the URL, not the declaration.",
                capability::download_backends()
            )));
        }
    }

    if o.contains("system") {
        if let Some(backend) = backend.filter(|b| !capability::accepts_system(b)) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`@system` is not an option on `{}`", backend),
            )
            .with_hint(format!(
                "it says this line may install into an environment the operating system owns, \
                 which is only a decision where the OS claims one: {}. Everywhere else there is \
                 nothing to override.",
                capability::system_backends()
            )));
        }
    }

    if o.contains("unverified") {
        if let Some(backend) = backend.filter(|b| !capability::accepts_unverified(b)) {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`@unverified` is not an option on `{}`", backend),
            )
            .with_hint(format!(
                "it says nothing checked the bytes, which is only a decision where something \
                 would have: {}. Everywhere else the package manager's own signed index \
                 answers for them.",
                capability::unverified_backends()
            )));
        }
    }

    // `@asset=all` installs every match, so there is no single artifact for one hash to cover.
    // Checked before the pinned-format rule below: both objections are true of
    // `@asset=all,sha256=…`, and this one names the reason the line cannot be fixed by
    // pinning a format.
    if o.one("asset")
        .is_some_and(|a| a.eq_ignore_ascii_case("all"))
        && o.contains("sha256")
    {
        return Err(GrammarError::new(
            origin.clone(),
            "`@asset=all` and `@sha256=` cannot both be set",
        )
        .with_hint(
            "`all` installs several files and one hash cannot verify them. Pin one file, or \
             drop the checksum.",
        ));
    }

    // One hash cannot cover an asset that varies by machine (D6): a shared module says
    // `github:x/y` and the Debian box downloads the `.deb` while the Fedora box downloads the
    // `.rpm`. A hand-written hash is only a claim about a file when the line names one file,
    // so it is legal only where the format is pinned to exactly one. Everywhere else the hash
    // is generated content and lives in `locks/<backend>.toml`.
    if o.contains("sha256")
        && backend.is_some_and(capability::selects_artifacts)
        && o.all("formats").len() != 1
    {
        let said = o.all("formats").len();
        return Err(GrammarError::new(
            origin.clone(),
            format!(
                "`@sha256` needs the line to pin exactly one format, and it {}",
                if said == 0 {
                    "pins none".to_string()
                } else {
                    format!("lists {}", said)
                }
            ),
        )
        .with_hint(
            "one release ships several files and one hash cannot verify them all. Add \
             `@formats=` naming one, or drop the checksum — Shall records the hash of what it \
             downloaded in `locks/` either way.",
        ));
    }

    Ok(())
}

/// Option rules from II.2's table that are about the options themselves rather than any
/// one backend.
fn validate_options(origin: &Origin, decl: &PackageDecl, absent: bool) -> Result<()> {
    let o = &decl.options;

    // II.2's table is the whole list. An unknown key used to be kept and handed downstream,
    // where something might act on it — `@lease=2h` is the one that mattered: II.16 retired
    // it, nothing writes it, and `StateRegistry::add` still read it and turned it into a
    // real expiry. So a key this document deleted was silently still a package that
    // uninstalls itself (S19). An option nobody reads is a line that does nothing; an
    // option someone still reads is worse.
    for key in o.keys() {
        if is_package_option_key(key) {
            // Same single-value refusal the typed lines get, one gate earlier than any
            // reader's `one()` could demote a repeat silently. (`@requires=` stays a list;
            // it is not in [`SINGLE_VALUE_OPTION_KEYS`].)
            if SINGLE_VALUE_OPTION_KEYS.contains(&key) && o.all(key).len() > 1 {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`@{key}` takes one value, given {}", o.all(key).len()),
                )
                .with_hint(
                    "a repeated single-value key keeps only its first — write the one you mean.",
                ));
            }
            continue;
        }
        let mut err = GrammarError::new(origin.clone(), format!("`@{}` is not an option", key));
        err = match key {
            // The one worth naming, because it used to work.
            "lease" | "duration" => err.with_hint(
                "a lease is a dated line now: `@expires=2026-07-17T14:00`. A file cannot hold \
                 \"2 hours\" — it would mean something different every time it was read.",
            ),
            _ => err.with_hint(format!(
                "options on a package are: {}, and the `*_install` hooks.",
                PACKAGE_OPTION_KEYS.join(", ")
            )),
        };
        return Err(err);
    }

    // A health check decides whether the machine is rolled back (XIII.5), so a line whose
    // check cannot be understood must not parse. `@health=port:donkey` that read as a shell
    // command would be a probe that fails every time and reverts every sync.
    if let Some(written) = o.one("health") {
        if crate::model::health::Probe::parse(written).is_none() {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`@health={}` is not a check", written),
            )
            .with_hint(
                "a health check is `port:8080` — something must be listening — or a command \
                 that exits 0, written plainly or as `cmd:systemctl is-active nginx`.",
            ));
        }
    }

    // `@hold` says "never upgrade this"; `@version=` says "this exact version". Together
    // they are a contradiction, not a refinement: hold means whatever is installed, and
    // version means something specific that may not be it. **Judged by VALUE, not presence**:
    // every consumer reads what `hold` SAYS, so `@hold=false` — "do not hold" — beside a
    // version is a line that means one thing, and refusing it with advice about keeping
    // whatever is installed misread the line to its author.
    let holds = o.one("hold").map(|v| {
        !v.is_empty() && !v.eq_ignore_ascii_case("false") && !v.eq_ignore_ascii_case("no")
    });
    if holds == Some(true) && o.contains("version") {
        return Err(GrammarError::new(
            origin.clone(),
            "`@hold` and `@version=` contradict each other",
        )
        .with_hint(
            "`@hold` keeps whatever is installed; `@version=` pins a specific one. Pick one.",
        ));
    }

    // `requires` is install ORDERING for things outside a package manager's own dependency
    // graph (V.29). A bare name would have to be resolved via `priority`, and the whole
    // point is that these are things with no one to ask.
    for req in o.all("requires") {
        if !req.contains(':') {
            return Err(GrammarError::new(
                origin.clone(),
                format!("`requires = {}` is a bare name", req),
            )
            .with_hint("`requires` needs a backend: `requires = apt:libfoo`."));
        }
    }

    // II.2: `expires` and `until` are absolute datetimes. A duration cannot work in a file
    // — the machine reading it next week has no idea when you wrote it (V.38), which is
    // exactly why `@lease=2h` was inert.
    for key in ["expires", "until"] {
        if let Some(v) = o.one(key) {
            if !is_absolute_datetime(v) {
                return Err(GrammarError::new(
                    origin.clone(),
                    format!("`@{}={}` is not an absolute date and time", key, v),
                )
                .with_hint(
                    "write it out in full: `@expires=2026-07-17T14:00`. A duration cannot \
                     work in a file — whoever reads it later has no idea when you wrote it.",
                ));
            }
        }
    }

    // `until` is the mirror of `expires` and only makes sense on `absent:` (absent now,
    // present after). On a present line it would mean "install this later", which the
    // grammar has no way to act on — so it is refused there, naming the file and line,
    // rather than parsed and quietly ignored.
    if !absent && o.contains("until") {
        return Err(
            GrammarError::new(origin.clone(), "`@until` is only for `absent:` lines").with_hint(
                "`@until` lifts an `absent:` line on a date (absent now, present after). To make \
             a present line lapse on a date, use `@expires`.",
            ),
        );
    }

    validate_backend_options(origin, decl.backend.as_deref(), &decl.options)?;
    Ok(())
}

/// Accepts RFC3339 and the `YYYY-MM-DDTHH:MM` form II.2 uses in its example. Rejects
/// anything that reads as a duration.
fn is_absolute_datetime(v: &str) -> bool {
    if chrono::DateTime::parse_from_rfc3339(v).is_ok() {
        return true;
    }
    for fmt in ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M", "%Y-%m-%d"] {
        if chrono::NaiveDateTime::parse_from_str(v, fmt).is_ok()
            || chrono::NaiveDate::parse_from_str(v, fmt).is_ok()
        {
            return true;
        }
    }
    false
}

/// The `absent:`-style prefixes, for building an "unrecognised line" message that lists
/// what was expected. Derived from [`KEYWORDS`] rather than listed again — the copy that
/// used to live here knew six of the eleven.
pub fn known_prefixes() -> Vec<&'static str> {
    KEYWORDS
        .iter()
        .filter(|k| k.takes_colon())
        .map(|k| k.spelling)
        .collect()
}

/// Every reserved word and what it is, for anything that has to quantify over the language.
///
/// The bare word, never the spelling: a caller asking "what words does this grammar reserve"
/// wants `link`, and one asking "what prefixes are there" wants [`known_prefixes`]. Two
/// questions, two functions, one table — so the answer cannot drift the way the three copies
/// this table replaced did.
pub fn reserved_words() -> Vec<(&'static str, KeywordRole)> {
    KEYWORDS.iter().map(|k| (k.word(), k.role)).collect()
}
