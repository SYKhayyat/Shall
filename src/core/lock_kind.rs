//! What `lock` and `unlock` can freeze, one name per thing, and how a person selects a set.
//!
//! **There were three axes and there are nine kinds, because the granularity was already there.**
//! `scripts` was one word covering seven separately-approved things — a hook, an event hook, an
//! adapter file, an `exec:`, a `generate:`, a health-check command and a `vars` provider — each
//! of which already has its own identity in `locks/hooks.toml` and its own prefix in that
//! identity. Approving all seven to approve one was not a limitation of the ledger. It was a
//! limitation of the word.
//!
//! **Below the kind there is a sub-category, spelled `kind:qualifier`.** `versions:apt` is apt's
//! pins, `hooks:after_install` is one hook across every package, `events:before_sync` is one
//! event wherever it is declared. Four of the nine kinds have such a division and five do not —
//! an `exec:` script has no category above itself, only its own name — and asking for one where
//! none exists is refused rather than silently matching nothing.
//!
//! **Why the scope is a word and not a flag.** `--backend apt` reads well until you want
//! `--except`, and then it cannot be written at all: an exclusion is a list, and a flag applies
//! to the whole command. "Everything except cargo's pins" is `--except versions:cargo` and has
//! no spelling as a flag. One mechanism that works in an include list, an exclusion and
//! `preferences.toml` identically beats two that each work in one place.

use crate::core::{Error, Result};
use std::fmt;

/// One freezable thing.
///
/// The variants are the vocabulary: these strings appear in `shall lock`, in `shall unlock`, in
/// `--except`, and in `[lock] freeze` / `[lock] except`. One list, so a name that works in one
/// place works in all four — the alternative is a config vocabulary that drifts from the CLI's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LockKind {
    /// Package version pins — `locks/versions.json`.
    Versions,
    /// Which manager each unpinned bare name resolved to — `locks/bare.HOST.toml`.
    Backends,
    /// Lifecycle hooks, keyed `<hook>:<package>`.
    Hooks,
    /// Hooks on Shall's own events, keyed `event:<event>@<origin>`.
    Events,
    /// Files under `adapters/`, keyed `adapters:<filename>`.
    Adapters,
    /// `exec:` scripts, keyed `exec:<script>`.
    Exec,
    /// `generate:` commands, keyed `generate:<command>`.
    Generate,
    /// Declared health-check commands, keyed `health:<command>`.
    Health,
    /// The `vars` provider, keyed `vars:<filename>`.
    Vars,
}

/// Every kind, in the order help text and listings print them: the two that describe packages,
/// then the seven that approve code.
pub const ALL: [LockKind; 9] = [
    LockKind::Versions,
    LockKind::Backends,
    LockKind::Hooks,
    LockKind::Events,
    LockKind::Adapters,
    LockKind::Exec,
    LockKind::Generate,
    LockKind::Health,
    LockKind::Vars,
];

/// The seven kinds that live in the approval ledger and gate something the config can execute.
pub const SCRIPTS: [LockKind; 7] = [
    LockKind::Hooks,
    LockKind::Events,
    LockKind::Adapters,
    LockKind::Exec,
    LockKind::Generate,
    LockKind::Health,
    LockKind::Vars,
];

/// The two kinds that describe packages rather than code.
pub const PACKAGES: [LockKind; 2] = [LockKind::Versions, LockKind::Backends];

impl LockKind {
    /// The one spelling. Parsing and printing read the same table, so a kind cannot be
    /// accepted under a name nothing prints.
    pub fn as_str(self) -> &'static str {
        match self {
            LockKind::Versions => "versions",
            LockKind::Backends => "backends",
            LockKind::Hooks => "hooks",
            LockKind::Events => "events",
            LockKind::Adapters => "adapters",
            LockKind::Exec => "exec",
            LockKind::Generate => "generate",
            LockKind::Health => "health",
            LockKind::Vars => "vars",
        }
    }

    /// What `kind:qualifier` means for this kind, in words a refusal can print, or `None` where
    /// the kind has no division below itself.
    ///
    /// Five kinds return `None` and that is not an oversight: an `exec:` script, an adapter
    /// file, a health command, a generator and the `vars` provider are each a flat set of items.
    /// Their granularity is the item's own name, which is what the positional `NAME…` argument
    /// has always selected.
    pub fn subdivision(self) -> Option<&'static str> {
        match self {
            LockKind::Versions | LockKind::Backends => Some("a package manager"),
            LockKind::Hooks => Some("a hook name, such as `after_install`"),
            LockKind::Events => Some("an event name, or the origin it is declared in"),
            LockKind::Adapters
            | LockKind::Exec
            | LockKind::Generate
            | LockKind::Health
            | LockKind::Vars => None,
        }
    }

    /// The prefix this kind's entries carry in `locks/hooks.toml`, for the six whose identity
    /// begins with a fixed word.
    ///
    /// `Hooks` is `None` and that is the interesting case: a lifecycle hook is keyed
    /// `<hook_name>:<package>` — `after_install:nginx` — so its first segment is the hook's own
    /// name and there is no constant to match. It is recognised by elimination, which is only
    /// safe because this list and [`ALL`] are checked against each other.
    pub fn ledger_prefix(self) -> Option<&'static str> {
        match self {
            LockKind::Events => Some("event"),
            LockKind::Adapters => Some("adapters"),
            LockKind::Exec => Some("exec"),
            LockKind::Generate => Some("generate"),
            LockKind::Health => Some("health"),
            LockKind::Vars => Some("vars"),
            LockKind::Hooks | LockKind::Versions | LockKind::Backends => None,
        }
    }

    /// Which kind an approval-ledger id belongs to.
    ///
    /// Anything whose prefix is not one of the six named ones is a lifecycle hook, because that
    /// is the one identity built from the hook's own name.
    /// `every_ledger_prefix_is_claimed_by_exactly_one_kind` fails if a seventh prefix is ever
    /// added without a kind, which is what keeps the by-elimination step honest.
    pub fn of_ledger_id(id: &str) -> LockKind {
        let head = id.split_once(':').map_or(id, |(h, _)| h);
        SCRIPTS
            .iter()
            .copied()
            .find(|k| k.ledger_prefix() == Some(head))
            .unwrap_or(LockKind::Hooks)
    }

    /// The sub-category an entry of this kind falls into, given its ledger id (or, for
    /// `Backends`, the manager the name resolved to — the one case where the answer is in the
    /// value rather than the key).
    ///
    /// `None` means "this entry has no sub-category", which a qualified target never matches.
    pub fn sub_of(self, key: &str, resolved_backend: Option<&str>) -> Option<String> {
        match self {
            // `apt:curl` — the manager is the head.
            LockKind::Versions => key.split_once(':').map(|(m, _)| m.to_string()),
            // A bare-name entry is keyed by the name; the manager is what it resolved to.
            LockKind::Backends => resolved_backend.map(str::to_string),
            // `after_install:nginx` — the hook's own name is the head.
            LockKind::Hooks => key.split_once(':').map(|(h, _)| h.to_string()),
            // `event:before_sync@repo` — matched on either half below.
            LockKind::Events => key
                .strip_prefix("event:")
                .map(|rest| rest.split_once('@').map_or(rest, |(e, _)| e).to_string()),
            _ => None,
        }
    }

    /// The second thing an `events:` qualifier may name: where the hook is declared.
    fn event_origin(key: &str) -> Option<String> {
        key.strip_prefix("event:")
            .and_then(|rest| rest.split_once('@'))
            .map(|(_, origin)| origin.to_string())
    }
}

impl fmt::Display for LockKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One selected thing: a kind, optionally narrowed to a sub-category, optionally subtracted.
///
/// `negated` is a field rather than a marker inside `qualifier`, because a qualifier holds text
/// the user typed and any sentinel put in there is a string a user can also type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LockTarget {
    pub kind: LockKind,
    pub qualifier: Option<String>,
    pub negated: bool,
}

impl LockTarget {
    fn whole(kind: LockKind) -> Self {
        Self {
            kind,
            qualifier: None,
            negated: false,
        }
    }

    /// Whether this target admits an entry of its kind. An unqualified target admits every
    /// entry; a qualified one admits those in its sub-category.
    fn admits(&self, key: &str, resolved_backend: Option<&str>) -> bool {
        let Some(want) = &self.qualifier else {
            return true;
        };
        if self.kind.sub_of(key, resolved_backend).as_deref() == Some(want.as_str()) {
            return true;
        }
        // An `events:` qualifier names the event or the origin. U15 keeps those separately
        // approved precisely because they are separate decisions, so both are addressable.
        self.kind == LockKind::Events && LockKind::event_origin(key).as_deref() == Some(want)
    }
}

impl fmt::Display for LockTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.qualifier {
            Some(q) => write!(f, "{}:{}", self.kind, q),
            None => write!(f, "{}", self.kind),
        }
    }
}

/// One word a person may type: a kind, or a group standing for several.
fn expand(word: &str) -> Option<Vec<LockKind>> {
    match word {
        "everything" => Some(ALL.to_vec()),
        "scripts" => Some(SCRIPTS.to_vec()),
        "packages" => Some(PACKAGES.to_vec()),
        other => ALL
            .iter()
            .copied()
            .find(|k| k.as_str() == other)
            .map(|k| vec![k]),
    }
}

/// The words a refusal offers back. Groups first, because a person who typed a wrong word
/// usually wanted a group.
fn vocabulary() -> String {
    let kinds: Vec<&str> = ALL.iter().map(|k| k.as_str()).collect();
    format!(
        "groups: everything, packages, scripts\n  kinds:  {}",
        kinds.join(", ")
    )
}

/// Parse one comma-separated word into targets, resolving groups and `kind:qualifier`.
fn parse_word(word: &str, flag: &str) -> Result<Vec<LockTarget>> {
    let (head, qualifier) = match word.split_once(':') {
        Some((h, q)) => (h.trim(), Some(q.trim())),
        None => (word, None),
    };

    let Some(kinds) = expand(head) else {
        return Err(Error::Validation(format!(
            "{flag}`{head}` is not something Shall can freeze.\n  {}",
            vocabulary()
        )));
    };

    let Some(want) = qualifier else {
        return Ok(kinds.into_iter().map(LockTarget::whole).collect());
    };
    if want.is_empty() {
        return Err(Error::Validation(format!(
            "{flag}`{head}:` names no sub-category. Write `{head}` for all of it, or \
             `{head}:something` for part."
        )));
    }

    // A qualifier on a group narrows every member that HAS that division and drops the rest:
    // `packages:apt` is apt's pins and apt's resolutions, and says nothing about `exec`.
    let divisible: Vec<LockKind> = kinds
        .iter()
        .copied()
        .filter(|k| k.subdivision().is_some())
        .collect();
    if divisible.is_empty() {
        return Err(Error::Validation(format!(
            "{flag}`{head}` has no sub-category, so `{head}:{want}` selects nothing. Every \
             {head} entry is\n  addressed by its own name instead: `shall lock {head} {want}`."
        )));
    }

    Ok(divisible
        .into_iter()
        .map(|kind| LockTarget {
            kind,
            qualifier: Some(want.to_string()),
            negated: false,
        })
        .collect())
}

/// A set of targets, however it was spelled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockSelection {
    targets: Vec<LockTarget>,
    /// Whether the user named anything explicitly, as opposed to taking the default. Config
    /// narrowing applies to the default and never to an explicit request: a `[lock] freeze` that
    /// could veto `shall lock exec` would make a command decline the one thing it was told.
    explicit: bool,
}

impl LockSelection {
    /// Everything, as a bare `shall lock` means it.
    pub fn everything() -> Self {
        Self {
            targets: ALL.iter().copied().map(LockTarget::whole).collect(),
            explicit: false,
        }
    }

    /// Parse `what` (comma-separated words, or `everything`) minus `except`.
    ///
    /// **An exclusion that selects nothing is an error, not an empty run.** `--except everything`
    /// and `lock exec --except exec` both describe a command with no work in it, and a `lock`
    /// that silently freezes nothing is indistinguishable from one that worked.
    pub fn parse(what: &str, except: &[String]) -> Result<Self> {
        let explicit = what != "everything";
        let mut targets: Vec<LockTarget> = Vec::new();
        for word in what.split(',').map(str::trim).filter(|w| !w.is_empty()) {
            for target in parse_word(word, "")? {
                if !targets.contains(&target) {
                    targets.push(target);
                }
            }
        }
        if targets.is_empty() {
            return Err(Error::Validation(format!(
                "nothing was named to freeze.\n  {}",
                vocabulary()
            )));
        }

        for word in except {
            for drop in parse_word(word.trim(), "--except ")? {
                if drop.qualifier.is_none() {
                    // `--except exec` removes the kind outright.
                    targets.retain(|t| t.kind != drop.kind);
                } else if targets.iter().any(|t| t.kind == drop.kind) {
                    // `--except versions:cargo` narrows a kind that is still wanted, so it is
                    // recorded as a subtraction the matcher consults rather than a removal.
                    targets.push(LockTarget {
                        negated: true,
                        ..drop
                    });
                }
            }
        }
        if targets.iter().all(|t| t.negated) {
            return Err(Error::Validation(
                "every kind was excluded, so this command would freeze nothing. Drop an \
                 `--except`, or name what you do want."
                    .to_string(),
            ));
        }

        targets.sort();
        Ok(Self { targets, explicit })
    }

    /// Narrow by a machine's standing preference, which applies only when the user named nothing.
    pub fn narrowed_by_config(mut self, allowed: &[LockKind]) -> Self {
        if !self.explicit {
            self.targets.retain(|t| allowed.contains(&t.kind));
        }
        self
    }

    /// Whether any work of this kind is selected at all — the question a caller asks before
    /// running that kind's approver.
    pub fn includes(&self, kind: LockKind) -> bool {
        self.targets.iter().any(|t| t.kind == kind && !t.negated)
    }

    /// Whether this specific entry is selected. `resolved_backend` is only consulted for
    /// `Backends`, whose sub-category lives in the value rather than the key.
    ///
    /// **An exclusion beats an inclusion**, so `everything --except versions:cargo` drops
    /// cargo's pins while keeping every other manager's. The other order would make the
    /// exclusion unwritable, since `everything` admits the entry first.
    pub fn admits(&self, kind: LockKind, key: &str, resolved_backend: Option<&str>) -> bool {
        let mine = || self.targets.iter().filter(|t| t.kind == kind);
        if mine()
            .filter(|t| t.negated)
            .any(|t| t.admits(key, resolved_backend))
        {
            return false;
        }
        mine()
            .filter(|t| !t.negated)
            .any(|t| t.admits(key, resolved_backend))
    }

    /// Whether **every** entry of this kind is selected: chosen with no qualifier narrowing it,
    /// and nothing subtracted from it.
    ///
    /// The question a caller asks before taking the rebuild-the-whole-file shortcut instead of
    /// merging entry by entry. `everything --except versions:cargo` still *includes* versions
    /// and must not take that shortcut, which is the case a plain "is this kind selected" check
    /// gets wrong.
    pub fn takes_all_of(&self, kind: LockKind) -> bool {
        let mut whole = false;
        for target in self.targets.iter().filter(|t| t.kind == kind) {
            if target.negated {
                return false;
            }
            whole |= target.qualifier.is_none();
        }
        whole
    }

    /// Whether every one of `set` is selected whole — the question the group-level reports ask
    /// before printing a summary that speaks for all of them.
    pub fn includes_all_whole(&self, set: &[LockKind]) -> bool {
        set.iter().all(|k| {
            self.targets
                .iter()
                .any(|t| t.kind == *k && t.qualifier.is_none() && !t.negated)
        })
    }

    pub fn is_empty(&self) -> bool {
        self.targets.iter().all(|t| t.negated)
    }

    pub fn targets(&self) -> &[LockTarget] {
        &self.targets
    }

    /// Every manager named as a qualifier **on `backends:`**, for validating them against the
    /// registry before a typo scopes the run to nothing (`Q9`, which `upgrade --backend aptt`
    /// already cost once).
    ///
    /// **`versions:` is deliberately not asked**, and the reason is the same one that keeps the
    /// name check off that kind: `locks/versions.json` travels between machines, so
    /// `versions:apt` is an ordinary scope on a host with no apt — a repo shared with a Debian
    /// box records apt pins, and a Windows machine has every right to release them. The bare
    /// lock is the opposite: it records what *this* host resolved, so a manager it does not run
    /// is a typo and nothing else. A typo on `versions:` still gets an answer, from the kind's
    /// own "nothing matched" warning, which quotes the scope back.
    pub fn managers_named(&self) -> Vec<String> {
        self.targets
            .iter()
            .filter(|t| t.kind == LockKind::Backends)
            .filter_map(|t| t.qualifier.clone())
            .collect()
    }
}

impl fmt::Display for LockSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<String> = self
            .targets
            .iter()
            .filter(|t| !t.negated)
            .map(|t| t.to_string())
            .collect();
        f.write_str(&names.join(", "))
    }
}
