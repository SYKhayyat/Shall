//! The removal guard: the last check between a plan and a purged system.
//!
//! Drift removal is derived from managed state, and managed state can be wrong â€” a
//! mis-scoped manifest, a bad adoption, a state file from another machine. When it is
//! wrong the planner does not produce a *small* mistake; it schedules every managed
//! package for removal and the engine carries it out one purge at a time.
//!
//! Two rules shape this module:
//!
//! 1. *Every* path that deletes is guarded, not just the reviewed ones. A guard on one
//!    command is a guard on nothing: the bug that motivated this arrived through `prune`,
//!    which nobody thought to check.
//! 2. `--yes` never overrides it. `-y` means "don't ask me questions", which every script
//!    and CI job passes; it must not also mean "yes, purge the system". The dedicated
//!    `--allow-mass-removal` is the only override, and it cannot be set permanently in
//!    config.

use crate::backends::BackendRegistry;
use crate::config::Config;
use crate::core::{Error, Result};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, warn};

/// Proof that this guard was consulted, and the only thing an effector will remove without.
///
/// **`README.md:358` promises that every path removing anything goes through one guard. Until
/// this type existed, that sentence was checked by a regex over source text** â€”
/// `removal_guard_enumeration_tests.rs`'s `is_removal_call`, matching `.remove(` with `sudo`
/// on the line, `.remove_repo(`, `.remove_shim(` and `.deprovision(`. `apply/firewall.rs` closes
/// a port with `deny_command`, which matches none of them, and the word `guard` appears nowhere
/// in that file. **The fix for `G-1` replaced a stale list of paths with a stale list of verbs**,
/// and the staleness moved into a predicate with a passing self-test, where nobody re-derives it.
///
/// So the enumeration is the compiler's now. Every effector that removes takes one of these, and
/// the only way to get one is to have asked. Effector six is covered by construction rather than
/// by someone remembering the list.
///
/// This is what `PlanScope` did for planning, applied to removal. `planner.rs` states the
/// technique better than this comment can: *"the case that reaps cannot be written without the
/// list that bounds it."*
///
/// **The private fields are load-bearing, and there is no public way around them.** `Reaped {}`
/// from outside this module is a compile error and there is no constructor but the guard's own
/// entry points, so a token cannot be minted by a caller who would rather not ask. This used to
/// say the same thing and then hand out `Reaped::for_reason(scope, "…")` on the next screen, with
/// the reason thrown away — so "cannot be minted" was delivered by a convention every caller
/// could decline, and 44 places did. **The reason string is gone with the constructor**, because a
/// reason that is not read is worse than no reason: it reads like an argument the guard
/// considered and the type is the argument.
#[derive(Debug, Clone, Copy)]
pub struct Reaped {
    /// Which command's removal this authorises. Carried so an effector can name it, and so the
    /// token is not silently reusable across two different commands' plans.
    scope: GuardScope,
    /// **How many removals the guard cleared**, which is the fact the token exists to carry. A
    /// caller cannot read "yes" out of it: `allowed()` answers zero for a token that cleared
    /// nothing, and the only way to get a non-zero one is to have asked with a real set.
    allowed: usize,
}

impl Reaped {
    /// What the guard was asked on behalf of.
    pub fn scope(&self) -> GuardScope {
        self.scope
    }

    /// How many removals this token carries. Zero means the guard cleared none, and a removal
    /// path that is about to remove something must refuse it (II.67's shape, one layer down).
    pub fn allowed(&self) -> usize {
        self.allowed
    }

    /// Whether this token authorises a removal of `count` things — the question a removal path
    /// asks immediately before it acts, and the one that makes "the guard ran" a fact about the
    /// size of the ask rather than a fact about having asked.
    pub fn authorises(&self, count: usize) -> bool {
        count > 0 && self.allowed >= count
    }
}

/// Which command is asking. Passed explicitly rather than inferred, so every caller has
/// to declare itself â€” a new deletion path cannot quietly inherit someone else's
/// exemption â€” and so a refusal can name what refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardScope {
    Apply,
    RemoveOrphans,
    PurgeUndeclared,
    Sync,
    Watch,
    Upgrade,
    Canary,
    Remove,
    ShellExit,
    ExpirySweep,
    Heal,
    Rebuild,
}

impl GuardScope {
    /// The command a user would recognize, for messages. It has to be what they typed:
    /// a refusal reading "prune refused" to someone running `purge-undeclared` names a
    /// command that does not exist, and gives them nothing to act on.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Apply => "apply",
            Self::RemoveOrphans => "remove-orphans",
            Self::PurgeUndeclared => "purge-undeclared",
            Self::Sync => "sync",
            Self::Watch => "watch",
            Self::Upgrade => "upgrade",
            Self::Canary => "upgrade --canary",
            Self::Remove => "uninstall",
            Self::ShellExit => "shell exit",
            Self::ExpirySweep => "expiry sweep",
            Self::Heal => "heal",
            Self::Rebuild => "rebuild",
        }
    }

    /// Whether a transaction run under this scope is **reconciling the machine against the
    /// manifest** â€” which decides whether its rollback may leave a removal in place (`U41`).
    ///
    /// A reconciling run removes a package *because nothing declares it*, and that fact is still
    /// true when the rollback fires. Putting it back hands the next sync the same work, which is
    /// the one mechanism in the program that provably un-converges.
    ///
    /// **Two scopes are not reconciling, and each has to be said out loud:**
    ///
    /// - `Rebuild` splits one operation into two transactions so a `Remove` and an `Install` of
    ///   the same package cannot race in one graph. Its removal phase is the first half of a
    ///   reinstall of *declared* packages; leaving one of those removals in place is not
    ///   convergence, it is a machine missing software it still declares.
    /// - `Remove` is a person typing `shall uninstall`. The removal was ordered by hand rather
    ///   than derived from a manifest, so a transaction that failed around it should give the
    ///   package back.
    ///
    /// Exhaustive, and deliberately so: a scope added later must say which it is, because
    /// inheriting `true` here means inheriting "a failed run may leave your software deleted".
    pub fn reconciles(&self) -> bool {
        match self {
            Self::Rebuild | Self::Remove => false,
            Self::Apply
            | Self::RemoveOrphans
            | Self::PurgeUndeclared
            | Self::Sync
            | Self::Watch
            | Self::Upgrade
            | Self::Canary
            | Self::ShellExit
            | Self::ExpirySweep
            | Self::Heal => true,
        }
    }

    /// How a refusal names this run in prose â€” the `(refused during â€¦)` half of a message.
    ///
    /// Separate from [`as_str`](Self::as_str) because the two answer different questions.
    /// `as_str` is the command to *retype* with a flag on it, so it has to be what the user
    /// typed. This is what the reader needs to understand the refusal, and there the
    /// difference that matters is **whether anybody was there** â€” `N7` makes an unattended
    /// `watch` tick revert by default, so "refused during watch" is the one phrasing that
    /// buries the fact worth reporting.
    ///
    /// Written out per variant rather than falling back to `as_str` for the rest: a catch-all
    /// arm here is how the label this replaced came to answer `"sync"` for nine of twelve
    /// scopes.
    pub fn during(&self) -> &'static str {
        match self {
            Self::Apply => "an apply",
            Self::RemoveOrphans => "remove-orphans",
            Self::PurgeUndeclared => "purge-undeclared",
            Self::Sync => "sync",
            Self::Watch => "an unattended watch tick",
            Self::Upgrade => "an upgrade",
            Self::Canary => "a canary upgrade",
            Self::Remove => "an uninstall",
            Self::ShellExit => "a shell exit",
            Self::ExpirySweep => "an expiry sweep",
            Self::Heal => "a recovery run",
            Self::Rebuild => "a rebuild",
        }
    }
}

/// Why a single package may not be removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protection {
    /// A `protected_packages` rule matched. Carries the rule so a refusal can cite it.
    Rule(String),
    /// The backend reports the OS itself treats this as essential.
    OsEssential(String),
    /// The manager reports a name that cannot be written as a package line, so Shall can
    /// never declare it â€” and what it cannot be asked to keep, it must not take away.
    Undeclarable,
    /// The name declares a resource, not a package: a `service:`, `link:` or `setting:`.
    /// `purge-undeclared` deletes packages nobody declared; a service nobody declared is not
    /// something that was installed, it is the state the machine is already in.
    NotAPackage(String),
}

impl Protection {
    pub fn reason(&self) -> String {
        match self {
            Self::Rule(rule) => format!("protected by config rule `{}`", rule),
            Self::OsEssential(backend) => {
                format!("{} reports it as essential to the system", backend)
            }
            Self::Undeclarable => {
                "its manager reports a name no line can hold, so Shall cannot manage \
                 it â€” and removing what you cannot declare is not something you asked for"
                    .to_string()
            }
            Self::NotAPackage(backend) => format!(
                "`{}:` declares a state, not an installed package â€” undeclaring one is a \
                 `sync` of a line you deleted, never a sweep of what you never declared",
                backend
            ),
        }
    }
}

/// The single decision function: may `name` be removed from `backend`?
///
/// Everything that asks "is this protected?" must route through here â€” the `protected`
/// command included. When the inspector and the enforcer answer separately they drift
/// apart, and an inspector that contradicts the guard is worse than none, because it is
/// believed.
///
/// `backend` is `None` when the caller does not know one â€” `shall protected jq`, where the user
/// named no manager. The config rules match on the name alone and are answered; the OS's
/// essential list is keyed by `backend:name` and cannot be, so it is not consulted. **An unknown
/// backend is this case, never the empty string**: `is_declarable("", "jq")` builds the line
/// `:jq`, which no grammar accepts, so every bare name came back `Undeclarable` before a rule
/// was read.
///
/// `os_essential` holds `backend:name` keys the OS flagged; pass an empty set when that
/// is unknown or irrelevant.
pub fn protection_of(
    config: &Config,
    backend: Option<&str>,
    name: &str,
    os_essential: &HashSet<String>,
) -> Option<Protection> {
    // Before the escape hatch, because neither of these is a policy: a name no line can hold
    // cannot be declared, so Shall never manages it and `unprotected_packages` has nothing
    // to release. Saying yes here would let `purge-undeclared` remove programs that could
    // never have been adopted in the first place.
    //
    // A resource is refused for a different reason, said out loud because it used to be an
    // accident: the declarability test asked whether a *package* line could hold the name,
    // `service:AppMgmt` is not a package line, and so every running service was refused by a
    // check that was not about services and printed a sentence that was false of them. The
    // moment that sentence was corrected â€” and it was false, `service:AppMgmt` parses â€” the
    // refusal would have evaporated and `purge-undeclared` could stop and disable every
    // service on the machine. This is that refusal, made on purpose.
    //
    // By backend, not by round-tripping the name: a `setting:` needs `@value=` before any line
    // will hold it, so the name alone would come back undeclarable and print the same false
    // sentence one backend over.
    if let Some(b) = backend {
        if crate::config::grammar::Statement::RESOURCE_BACKENDS.contains(&b) {
            return Some(Protection::NotAPackage(b.to_string()));
        }
    }
    if !crate::config::grammar::is_declarable(backend, name) {
        return Some(Protection::Undeclarable);
    }

    // One question, one pass. An explicit un-protect entry wins over everything, including
    // the OS's own essential flag â€” the user saying "I know, I manage this one myself", and
    // nothing should overrule that, or the escape hatch does not open for exactly the packages
    // someone would need it for. Asking it as two calls meant `protection_rule` re-scanned
    // `unprotected_packages` after `unprotect_rule` had already answered `None` over the same
    // list with the same input.
    match config.protection_of(name) {
        crate::config::ProtectionAnswer::Unprotected(_) => return None,
        crate::config::ProtectionAnswer::Protected(rule) => {
            return Some(Protection::Rule(rule.to_string()))
        }
        crate::config::ProtectionAnswer::Neither => {}
    }
    if let Some(b) = backend {
        if os_essential.contains(&format!("{}:{}", b, name)) {
            return Some(Protection::OsEssential(b.to_string()));
        }
    }
    None
}

/// A removal the guard objects to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Objection {
    Protected {
        key: String,
        reason: String,
    },
    TooMany {
        count: usize,
        limit: usize,
        /// The `[guard]` key that set `limit`, so the refusal names the number the reader has
        /// to change. Two ceilings answer this objection now (`Y20`) and a message reading
        /// `max_removals` about a port closure sends the reader to the wrong line.
        setting: &'static str,
    },
    /// The plan installs more packages at once than `max_installs` allows (II.10). The
    /// install-side twin of `TooMany`: a mis-globbed manifest schedules a flood of
    /// installs, and the count is the fact that explains it.
    TooManyInstalls {
        count: usize,
        limit: usize,
    },
    /// A desired package is on the `deny_packages` list (II.10) â€” never install this.
    Denied {
        key: String,
    },
    /// `pinned_only` is set and a desired package has no explicit `@version=` (II.10).
    Unpinned {
        key: String,
    },
    /// The backend this removal goes through could not report which packages the OS needs,
    /// so the removal cannot be checked against OS-essentials. Protection-class: a mass
    /// flag answers the count and nothing else, so nothing clears this but the manager
    /// answering.
    UnverifiedEssentials {
        key: String,
        backend: String,
    },
}

/// The guard's verdict over a removal set.
#[derive(Debug, Default, Clone)]
pub struct GuardReport {
    pub objections: Vec<Objection>,
    /// The objections a mass flag answered, kept rather than dropped on the floor.
    ///
    /// **A report that only subtracts cannot say what it let through.** Clearing an objection
    /// and never having raised one produce the same `objections` list, so the only difference
    /// between "the flag was needed" and "the flag was idle" used to be a `warn!` line â€” and a
    /// line is something a caller has to overhear rather than something it can read. Recorded
    /// here so [`allow_the_count`] announces a value it holds.
    pub allowed_by_flag: Vec<Objection>,
}

/// How many individual packages a refusal names before summarizing. A mass-removal plan
/// would otherwise print hundreds of lines above the one number that explains it.
///
/// **`pub` so the guard's tests can reach it.** They live in `tests/app/sync/guard/`, which
/// links the library without `cfg(test)`, and a private item is invisible from there.
pub const MAX_LISTED: usize = 10;

impl GuardReport {
    pub fn is_empty(&self) -> bool {
        self.objections.is_empty()
    }

    /// A refusal that says what is wrong and how to proceed. Leads with the count, since
    /// that is the fact that explains the rest.
    pub fn message(&self, scope: GuardScope, kind: RemovalKind) -> String {
        let mut out = format!("{}: refusing this removal.\n", scope.as_str());

        // Every count that objected, not the first: a set can be over its own ceiling and over
        // the total at once, and naming one of them sends a user to raise a number that leaves
        // them refused by the other.
        for o in &self.objections {
            if let Objection::TooMany {
                count,
                limit,
                setting,
            } = o
            {
                let (verb, noun) = counted_as(setting);
                out.push_str(&format!(
                    "  - it {} {} {}, over the limit of {} ([guard] {})\n",
                    verb, count, noun, limit, setting
                ));
            }
        }

        let protected: Vec<&Objection> = self
            .objections
            .iter()
            .filter(|o| matches!(o, Objection::Protected { .. }))
            .collect();
        for o in protected.iter().take(MAX_LISTED) {
            if let Objection::Protected { key, reason } = o {
                out.push_str(&format!("  - {} would be removed ({})\n", key, reason));
            }
        }
        if protected.len() > MAX_LISTED {
            out.push_str(&format!(
                "  - â€¦and {} more protected {}(s)\n",
                protected.len() - MAX_LISTED,
                match kind {
                    RemovalKind::Package => "package",
                    RemovalKind::Extra => "resource",
                    RemovalKind::Port => "port",
                }
            ));
        }

        // A second listing loop, not a merged one: a protection rule and an unverifiable
        // manager are different facts with different remedies, and the reader should be able
        // to count them apart.
        let unverified: Vec<&Objection> = self
            .objections
            .iter()
            .filter(|o| matches!(o, Objection::UnverifiedEssentials { .. }))
            .collect();
        for o in unverified.iter().take(MAX_LISTED) {
            if let Objection::UnverifiedEssentials { key, backend } = o {
                out.push_str(&format!(
                    "  - {} would be removed, but `{}` cannot currently report which \
                     packages the OS needs\n",
                    key, backend
                ));
            }
        }
        if unverified.len() > MAX_LISTED {
            out.push_str(&format!(
                "  - â€¦and {} more through managers that could not answer\n",
                unverified.len() - MAX_LISTED
            ));
        }

        // The advice has to be executable. `shall unmanage` takes a package line, so offering
        // it for a `link:` teardown names a command that cannot accept the thing it is about;
        // for an extra the equivalent act is putting the declaration back.
        match kind {
            RemovalKind::Package => out.push_str(
                "\nThis usually means managed state has drifted from your manifests â€” run \
                 `shall plan` and read it before proceeding.\n\n\
                 What to do:\n  \
                 shall protected <pkg>          why a package is guarded\n  \
                 shall unmanage <pkg>           stop managing it WITHOUT uninstalling it\n  \
                 <command> --allow-mass-removal carry out this removal anyway\n  \
                 [guard] unprotected_packages    exempt a package permanently (preferences.toml)",
            ),
            RemovalKind::Extra => out.push_str(
                "\nThese are resources a declaration put in place â€” a `link:`, `service:`, \
                 `setting:`, `shim:`, `schedule:` or `repo:` line that is no longer in any \
                 module. `sync` undoes what is no longer declared.\n\n\
                 What to do:\n  \
                 shall plan                     see exactly what would be undone\n  \
                 put the line back              if the deletion was not what you meant\n  \
                 <command> --allow-mass-removal carry out this teardown anyway\n  \
                 [guard] unprotected_packages    exempt one permanently (preferences.toml)",
            ),
            // Never `put the line back`: nobody declared these â€” that is *why* they are closing.
            // The act that keeps a port open is declaring it, which is the opposite instruction.
            RemovalKind::Port => out.push_str(
                "\nThese ports are open on the machine and no `firewall:` line declares them, \
                 so `sync` closes them (`N1`). A machine you reach over the network is a \
                 machine this can cut you off from â€” read the list before you clear it.\n\n\
                 What to do:\n  \
                 shall plan                     see exactly what would be closed\n  \
                 firewall:<port>/<proto>        declare a port you meant to keep open\n  \
                 <command> --allow-mass-removal close this many anyway\n  \
                 [guard] unprotected_packages    exempt one permanently (preferences.toml)",
            ),
        }
        out
    }
}

/// What backends answered about OS-essential packages â€” and which could not.
pub struct EssentialAnswers {
    /// `backend:name` pairs the running OS reports as essential, for the backends asked.
    pub names: HashSet<String>,
    /// Backends whose essential question has no answer this run. **A failure is an answer
    /// too**: a removal through one of these cannot be checked against what the OS needs,
    /// and the guard refuses it rather than reading the silence as "nothing here is
    /// essential" â€” that reading is how the safety rail fails open.
    pub unanswered: BTreeSet<String>,
}

/// What one backend answered about its OS-essential packages.
#[derive(Debug, Clone)]
pub enum EssentialOutcome {
    /// The manager answered; the vec holds raw names, qualified per backend when folded.
    Reported(Vec<String>),
    /// The manager is here and its query failed. Removals through it are refused this run.
    QueryFailed,
    /// No queryable manager exists here to ask (II.7c) — nothing installed through it on
    /// this machine, so there is no subject for the question and nothing to protect from.
    NothingToAsk,
}

/// One command's cache of essential answers, keyed by backend name.
///
/// The audit's arithmetic: the orphan path enforces the same removal set twice (`H8`'s ask +
/// spend split), `heal` once per interrupted entry, and each enforcement re-ran every
/// backend's `essential()` subprocess from scratch. One query per backend per command serves
/// them all; the caller owns the map, so a new command starts cold and `watch` ticks cannot
/// grow stale answers across passes.
pub type EssentialsCache = HashMap<String, EssentialOutcome>;

/// Names the OS itself reports as essential, per backend, for the backends being removed
/// from. Queried live so it tracks the running system rather than a list we maintain.
/// A backend whose query fails lands in [`EssentialAnswers::unanswered`] and blocks the
/// removals that would have needed it.
pub async fn essential_names(
    registry: &Arc<BackendRegistry>,
    backends: &HashSet<String>,
    max_parallel: usize,
    cache: &mut EssentialsCache,
) -> EssentialAnswers {
    // Each `essential()` is a subprocess and they have nothing to say to one another, so they
    // run at once — the UNCACHED ones do; anything this command already asked is served from
    // `cache` without a second subprocess.
    use futures::stream::StreamExt;
    let uncached: Vec<String> = backends
        .iter()
        .filter(|b| !cache.contains_key(*b))
        .cloned()
        .collect();
    let fresh = futures::stream::iter(uncached)
        .map(|name| {
            let registry = registry.clone();
            async move {
                // **Two kinds of "cannot ask", and only one refuses.** A backend that is not
                // on this machine has nothing installed through it here (II.7c) — the
                // essential question has no subject, and the planner already declines those
                // removals upstream. A backend that IS here and whose query fails is the
                // dangerous one: silence must not read as "nothing here is essential".
                let Some(queryable) = registry
                    .get(&name)
                    .and_then(|backend| backend.as_queryable().cloned())
                else {
                    debug!(
                        "backend '{}' is not queryable here; no essential set exists to ask for.",
                        name
                    );
                    return (name, EssentialOutcome::NothingToAsk);
                };
                match queryable.essential().await {
                    Ok(names) => {
                        debug!(
                            "backend '{}' reports {} essential package(s).",
                            name,
                            names.len()
                        );
                        (
                            name.clone(),
                            EssentialOutcome::Reported(
                                names.iter().map(|n| format!("{}:{}", name, n)).collect(),
                            ),
                        )
                    }
                    Err(e) => {
                        warn!(
                            "backend '{}' could not report which packages the OS needs ({}); \
                             removals through it are refused this run.",
                            name, e
                        );
                        (name, EssentialOutcome::QueryFailed)
                    }
                }
            }
        })
        .buffer_unordered(max_parallel.max(1))
        .collect::<Vec<_>>()
        .await;

    for (backend, outcome) in &fresh {
        cache.insert(backend.clone(), outcome.clone());
    }
    // The answer folds from CACHE, not from the fresh batch: cached and fresh outcomes take
    // identical shape, so a second enforcement in one command sees exactly the sets the
    // first saw without re-running any subprocess.
    let mut answers = EssentialAnswers {
        names: HashSet::new(),
        unanswered: BTreeSet::new(),
    };
    for backend in backends {
        if let Some(outcome) = cache.get(backend) {
            match outcome {
                EssentialOutcome::Reported(qualified) => answers.names.extend(qualified.clone()),
                EssentialOutcome::QueryFailed => {
                    answers.unanswered.insert(backend.clone());
                }
                EssentialOutcome::NothingToAsk => {}
            }
        }
    }
    answers
}

/// Inspect a removal set and report what disqualifies it. `removals` are
/// `(backend, name)` pairs. An empty report means the plan may proceed.
pub async fn inspect(
    config: &Config,
    registry: &Arc<BackendRegistry>,
    removals: &[(String, String)],
) -> GuardReport {
    // A fresh ledger: this is the preview, and a preview that spent a budget would report a
    // different answer the second time it was asked.
    inspect_removals(
        config,
        registry,
        removals,
        RemovalKind::Package,
        &Reaping::new(),
        &mut EssentialsCache::default(),
    )
    .await
}

/// The one place the mass flags are honoured over a removal report, and the one thing they
/// clear.
///
/// II.10: a mass flag answers exactly one refusal â€” the count. It used to clear every objection,
/// so the flag meaning "yes, 50 packages is what I meant" also deleted python3. A confirmation
/// asks; a refusal says no, and protection is a refusal (V.26): nothing overrides it.
///
/// Every removal ceiling answers to `--allow-mass-removal`, because the question it answers is
/// one question (`Y20`). `max_total_changes` answers to that flag *and* to
/// `--allow-mass-install`, because a total is made of both (`N8`) â€” and to nothing else, which
/// is why this is a match on the setting rather than a blanket retain.
///
/// **What the flag answered is moved, not deleted.** This was a `retain` and a
/// `before != after` around the `warn!`, whose only observable was the line itself: flipped to
/// `==`, the announcement went to every run that did *not* need an override and was withheld
/// from the runs that did, while the report stayed byte for byte the same. A test that read the
/// `tracing` output was written for it and withdrawn the same day â€” callsite `Interest` is
/// cached globally and other tests create and drop dispatchers on other threads throughout the
/// run, so the capture came back empty in 2 of 3 identical Linux runs. Partitioning answers it
/// without a subscriber: the objections the flag cleared are on the report, so what was allowed
/// is a value a caller reads rather than a line it has to overhear.
///
/// **`pub` so the guard's tests can reach it.** They live in `tests/app/sync/guard/`, which
/// links the library without `cfg(test)`, and a private item is invisible from there.
pub fn allow_the_count(config: &Config, report: &mut GuardReport, scope: GuardScope) {
    if !config.allow_mass_removal && !config.allow_mass_install {
        return;
    }
    let (allowed, kept): (Vec<Objection>, Vec<Objection>) = std::mem::take(&mut report.objections)
        .into_iter()
        .partition(|o| match o {
            // `--allow-mass-install` clears the total, because a total is made of installs too,
            // and it clears nothing else: the flag that means "yes, install that many" must not
            // also answer "yes, remove that many". That conflation is II.10's whole point, one
            // ceiling up.
            Objection::TooMany { setting, .. } => {
                config.allow_mass_removal || *setting == TOTAL_KEY
            }
            _ => false,
        });
    report.objections = kept;
    if let Some(said) = announcement(&allowed, config, scope) {
        warn!("{}", said);
    }
    // Extended rather than assigned: nothing calls this twice on one report today, and a report
    // that forgot the first answer when asked a second time would be a silent one.
    report.allowed_by_flag.extend(allowed);
}

/// What a run says when a mass flag answered its count â€” `None` when none did.
///
/// A function returning the sentence rather than an `if` around the `warn!`, for the same reason
/// the objections are moved rather than dropped: a condition whose only consequence is whether a
/// log line exists can be reversed without failing anything, and the test that would catch it has
/// to subscribe to `tracing` from a binary where the callsite cache is shared with every other
/// test on every other thread.
///
/// **Both halves of the sentence are read off what happened, never off the caller.** The ceiling
/// comes from each objection's own `setting` â€” [`counted_as`], for the reason its own doc gives â€”
/// and the flags from the config. Written from the caller's noun and a hardcoded
/// `--allow-mass-removal`, it told a run that passed only `--allow-mass-install` that a *removal*
/// count had been allowed by a *removal* flag: a ceiling, a noun and a flag, none of which were
/// that run's. `shall protected` has always printed the true rule â€” *"either flag answers
/// `max_total_changes`"* â€” so the guard's own line was the one surface contradicting it (`J9`).
///
/// **`pub` so the guard's tests can reach it.** They live in `tests/app/sync/guard/`, which
/// links the library without `cfg(test)`, and a private item is invisible from there.
pub fn announcement(allowed: &[Objection], config: &Config, scope: GuardScope) -> Option<String> {
    let flags = flags_that_allowed(config)?;
    let counts: Vec<String> = allowed
        .iter()
        .filter_map(|o| match o {
            Objection::TooMany {
                count,
                limit,
                setting,
            } => {
                let (verb, noun) = counted_as(setting);
                Some(format!(
                    "{} {} {}, over the limit of {} ([guard] {})",
                    verb, count, noun, limit, setting
                ))
            }
            _ => None,
        })
        .collect();
    if counts.is_empty() {
        return None;
    }
    Some(format!(
        "'{}' {} â€” allowed by {}.",
        scope.as_str(),
        counts.join("; "),
        flags
    ))
}

/// The mass flags this run actually passed, as a user would type them â€” `None` when it passed
/// neither, because then nothing was allowed and the sentence has no subject.
///
/// Both are named when both were passed. Attributing one of them instead would mean deciding
/// which was load-bearing, and for `max_total_changes` â€” the one ceiling either flag answers
/// (`N8`) â€” either one of them was.
fn flags_that_allowed(config: &Config) -> Option<&'static str> {
    match (config.allow_mass_removal, config.allow_mass_install) {
        (true, true) => Some("--allow-mass-removal and --allow-mass-install"),
        (true, false) => Some("--allow-mass-removal"),
        (false, true) => Some("--allow-mass-install"),
        (false, false) => None,
    }
}

/// What is being taken away. Every kind answers to `protected_packages` and to a ceiling; they
/// differ in one check, in which ceiling, and in what a refusal tells you to do.
///
/// The package/extra distinction exists because [`protection_of`]'s declarability test asks
/// "could a package line ever have held this name?", and for an extra the answer is structurally
/// no â€” a `link:`/`service:`/`setting:` key is not a package line and never parses as one.
/// Running that test over an extra marks every extra `Undeclarable` and refuses every teardown
/// forever, which is a guard that has stopped being about the user's intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalKind {
    Package,
    /// A `link:`/`service:`/`setting:`/`shim:`/`schedule:`/`repo:` resource leaving the model.
    Extra,
    /// A port closed because no `firewall:` line declares it (`N8`).
    Port,
}

impl RemovalKind {
    /// The `[guard]` key a refusal about this kind of removal must name, because a refusal that
    /// does not say which setting to change is a refusal a user cannot answer.
    pub fn ceiling_key(self) -> &'static str {
        match self {
            Self::Package => "max_removals",
            Self::Extra => "max_extra_removals",
            Self::Port => "max_port_closures",
        }
    }

    fn ceiling(self, config: &Config) -> usize {
        match self {
            Self::Package => config.guard.max_removals,
            Self::Extra => config.guard.max_extra_removals,
            Self::Port => config.guard.max_port_closures,
        }
    }
}

/// The `[guard]` key for the ceiling over everything one command changes (`N8`).
///
/// **`pub` so the guard's tests can reach it.** They live in `tests/app/sync/guard/`, which
/// links the library without `cfg(test)`, and a private item is invisible from there.
pub const TOTAL_KEY: &str = "max_total_changes";

/// What a ceiling counted, as the verb and noun a refusal says it with. Derived from the setting
/// rather than from the caller, so the sentence and the key it names cannot describe different
/// things â€” a message reading `removes 40 packages` above `[guard] max_port_closures` is worse
/// than no message.
fn counted_as(setting: &str) -> (&'static str, &'static str) {
    match setting {
        "max_removals" => ("removes", "packages"),
        "max_extra_removals" => ("removes", "managed resources"),
        "max_port_closures" => ("closes", "ports"),
        _ => ("makes", "changes in total"),
    }
}

/// What one command has taken away so far, so a ceiling is a budget for the command rather than
/// for each phase.
///
/// **A sync removes in four places** â€” the transaction's packages, the extras teardown, the
/// firewall's undeclared ports, and `repo remove` on the imperative path â€” and each used to
/// check its own list against the ceiling. `inspect_removals` took an `also_removing: usize` for
/// exactly this reason, which made the count something every caller assembled by hand: two
/// passed the right number and **`apply/firewall.rs` passed `0`**, so four packages and four
/// ports under a limit of five were invisible to every guard call in the run.
///
/// One value, owned by the command, incremented where the guard clears a removal. A caller
/// cannot pass the wrong number because a caller no longer passes a number.
///
/// **A count per kind, because there is a ceiling per kind** (`Y20`, `N8`). `max_removals` is
/// about software leaving the machine; `max_extra_removals` is about the resources a declaration
/// put in place; `max_port_closures` is about reachability. Sharing one budget makes the
/// strictest govern all of them, so a server whose first firewall declaration closes forty ports
/// could not also remove a package. All are answered by `--allow-mass-removal`, because "yes,
/// that many, I meant it" is one question.
///
/// **And one count of everything, because a command can pass every per-kind ceiling and still
/// do more than anyone meant** (`N8`). `additions` is what the per-kind ceilings do not cover â€”
/// installs and upgrades, resources created, ports opened â€” held here rather than anywhere else
/// because `max_total_changes` is a budget for the command, and the command is what owns this.
#[derive(Debug, Default)]
pub struct Reaping {
    packages: std::sync::atomic::AtomicUsize,
    extras: std::sync::atomic::AtomicUsize,
    ports: std::sync::atomic::AtomicUsize,
    additions: std::sync::atomic::AtomicUsize,
}

impl Reaping {
    pub fn new() -> Self {
        Self::default()
    }

    /// How many of this kind the command has already cleared.
    pub fn so_far(&self, kind: RemovalKind) -> usize {
        self.counter(kind)
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Everything the command has changed so far, of every kind â€” what `max_total_changes` is
    /// measured against.
    pub fn changes_so_far(&self) -> usize {
        use std::sync::atomic::Ordering::Relaxed;
        self.packages.load(Relaxed)
            + self.extras.load(Relaxed)
            + self.ports.load(Relaxed)
            + self.additions.load(Relaxed)
    }

    /// Record a cleared set. Called by the `enforce*` family and by nothing else: a removal
    /// counted without being checked would raise the total for everyone behind it while
    /// answering to nothing itself.
    fn record(&self, kind: RemovalKind, n: usize) {
        self.counter(kind)
            .fetch_add(n, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record changes that are not removals â€” installs and upgrades, resources created, ports
    /// opened. They answer to no ceiling of their own beyond `max_installs`, but they are
    /// changes, and `max_total_changes` counts changes.
    pub(crate) fn record_addition(&self, n: usize) {
        self.additions
            .fetch_add(n, std::sync::atomic::Ordering::Relaxed);
    }

    fn counter(&self, kind: RemovalKind) -> &std::sync::atomic::AtomicUsize {
        match kind {
            RemovalKind::Package => &self.packages,
            RemovalKind::Extra => &self.extras,
            RemovalKind::Port => &self.ports,
        }
    }
}

/// The `max_total_changes` objection, or `None` when the command still has room.
///
/// One function, called by every gate, because a total assembled at some gates and not others
/// is a total that reports whichever subset happened to run â€” the `also_removing: usize` shape
/// that `S55` was, one level up.
///
/// **`pub` so the guard's tests can reach it.** They live in `tests/app/sync/guard/`, which
/// links the library without `cfg(test)`, and a private item is invisible from there.
pub fn too_many_changes(config: &Config, reaping: &Reaping, adding: usize) -> Option<Objection> {
    let limit = config.guard.max_total_changes;
    let total = reaping.changes_so_far() + adding;
    (limit > 0 && total > limit).then_some(Objection::TooMany {
        count: total,
        limit,
        setting: TOTAL_KEY,
    })
}

/// The identities a `protected_packages` rule is matched against for one removal.
///
/// A package contributes its name and nothing else. An extra whose identity is a path also
/// contributes that path's final component, so `protected_packages = ["vimrc"]` protects
/// `link:/home/u/.vimrc` â€” a user names the thing, not the absolute path Shall happens to
/// key it by, and a rule that only matched the full path would silently protect nothing.
///
/// **`pub` so the guard's tests can reach it.** They live in `tests/app/sync/guard/`, which
/// links the library without `cfg(test)`, and a private item is invisible from there.
pub fn protected_names(kind: RemovalKind, name: &str) -> Vec<&str> {
    let mut names = vec![name];
    if kind == RemovalKind::Extra {
        if let Some(base) = name.rsplit(['/', '\\']).next() {
            if base != name && !base.is_empty() {
                names.push(base);
            }
        }
    }
    names
}

/// Inspect a removal set of one kind against its own ceiling and against the command's total,
/// reading what the command has already cleared off `reaping`.
///
/// **This is the pure half and it stays pure**: it reads the ledger and never writes to it, so a
/// preview may ask without spending anyone's budget. The `enforce*` family is what records.
/// Taking the ledger rather than a number is the point â€” `already: usize` was a parameter three
/// callers assembled by hand and one answered with a `0` (`S55`).
pub async fn inspect_removals(
    config: &Config,
    registry: &Arc<BackendRegistry>,
    removals: &[(String, String)],
    kind: RemovalKind,
    reaping: &Reaping,
    essentials: &mut EssentialsCache,
) -> GuardReport {
    let mut report = GuardReport::default();
    if removals.is_empty() {
        return report;
    }

    let (os_essential, unanswered) = match kind {
        RemovalKind::Package => {
            let backends: HashSet<String> = removals.iter().map(|(b, _)| b.clone()).collect();
            let answers =
                essential_names(registry, &backends, config.max_parallel, essentials).await;
            (answers.names, answers.unanswered)
        }
        // `service`/`link`/`setting` are not package managers and have no essential list to
        // ask for; querying them would be a round trip that can only return nothing. Neither is
        // a firewall.
        RemovalKind::Extra | RemovalKind::Port => (HashSet::new(), Default::default()),
    };

    for (backend, name) in removals {
        let protection = match kind {
            RemovalKind::Package => protection_of(config, Some(backend), name, &os_essential),
            RemovalKind::Extra | RemovalKind::Port => {
                protected_names(kind, name).into_iter().find_map(|n| {
                    config
                        .protection_rule(n)
                        .map(|r| Protection::Rule(r.to_string()))
                })
            }
        };
        if let Some(p) = protection {
            report.objections.push(Objection::Protected {
                key: format!("{}:{}", backend, name),
                reason: p.reason(),
            });
            // Already refused by name; a second objection about the same package is noise.
            continue;
        }
        if matches!(kind, RemovalKind::Package) && unanswered.contains(backend) {
            report.objections.push(Objection::UnverifiedEssentials {
                key: format!("{}:{}", backend, name),
                backend: backend.clone(),
            });
        }
    }

    let total = removals.len() + reaping.so_far(kind);
    let limit = kind.ceiling(config);
    if limit > 0 && total > limit {
        report.objections.push(Objection::TooMany {
            count: total,
            limit,
            setting: kind.ceiling_key(),
        });
    }
    // Both ceilings, when both are busted: a refusal naming one number a user raises, only to
    // meet the other on the next run, is a refusal that lied about what it wanted.
    report
        .objections
        .extend(too_many_changes(config, reaping, removals.len()));

    report
}

/// Enforce the guard for `scope`. `Ok(())` means the removal may proceed.
///
/// The override is `config.allow_mass_removal` (the `--allow-mass-removal` flag), never
/// `--yes`.
pub async fn enforce(
    config: &Config,
    registry: &Arc<BackendRegistry>,
    removals: &[(String, String)],
    reaping: &Reaping,
    scope: GuardScope,
) -> Result<Reaped> {
    enforce_kind(
        config,
        registry,
        removals,
        RemovalKind::Package,
        reaping,
        scope,
    )
    .await
}

/// Enforce the guard over the extras a sync is about to undo (`link:`, `service:`, `setting:`,
/// `shim:`, `schedule:`, `repo:`).
///
/// `also_removing` is the number of packages the same command already plans to remove, so the
/// ceiling is checked once against the whole command rather than once per phase.
///
/// This exists because the teardown loop in `app/apply/extras.rs` runs outside the transaction
/// and therefore outside the plan-time `enforce` that covers packages. Ten call sites can reach
/// a backend `remove`; that one was the only one no guard stood in front of, and a `link:` whose
/// target is a decrypted secret is not a smaller loss than a package.
pub async fn enforce_extras(
    config: &Config,
    registry: &Arc<BackendRegistry>,
    removals: &[(String, String)],
    reaping: &Reaping,
    scope: GuardScope,
) -> Result<Reaped> {
    enforce_kind(
        config,
        registry,
        removals,
        RemovalKind::Extra,
        reaping,
        scope,
    )
    .await
}

/// What `apply` would refuse about this plan, as messages, changing nothing.
///
/// **Here rather than in `verbs/plan.rs`, because the preview's only value is being the same
/// question.** The call site there ran one `inspect` over both removal lists, which put the
/// package rules over resource keys and predicted a refusal `apply` did not perform; then it
/// asked kind by kind with a hand-written `also_removing` for each. Both were the preview
/// drifting from the enforcer while looking like it agreed. One ledger, the kinds in the order
/// the engine gates them, and the numbers come off the ledger.
///
/// **Firewall ports are absent on purpose**: they are computed against the live machine at
/// apply time, so a plan file cannot hold them and this cannot count them. A perimeter change
/// can therefore push a run over `max_total_changes` that previewed clean â€” which is the same
/// thing `plan` has always said about a machine that moves under it.
pub async fn preview_refusals(
    config: &Config,
    registry: &Arc<BackendRegistry>,
    installs: usize,
    package_removals: &[(String, String)],
    extra_removals: &[(String, String)],
    scope: GuardScope,
) -> Vec<String> {
    let reaping = Reaping::new();
    // One cache across both phases: the same backend asked twice answers once.
    let mut cache = EssentialsCache::default();
    let mut refusals = Vec::new();
    // The engine's order: packages, then the install ceiling, then the extras teardown.
    for (pairs, kind) in [
        (package_removals, RemovalKind::Package),
        (extra_removals, RemovalKind::Extra),
    ] {
        let mut report =
            inspect_removals(config, registry, pairs, kind, &reaping, &mut cache).await;
        allow_the_count(config, &mut report, scope);
        if !report.is_empty() {
            refusals.push(report.message(scope, kind));
        }
        // Recorded whether or not it objected. A refused phase stops the engine, so the later
        // ones would never run â€” but a preview that reported the first refusal and went quiet
        // about the rest would have to be run once per fix.
        reaping.record(kind, pairs.len());
    }
    if let Err(e) = enforce_installs(config, installs, &reaping, scope).await {
        refusals.push(e.to_string());
    }
    refusals
}

/// Enforce the guard over the ports a sync is about to close because no `firewall:` line
/// declares them (`N8`).
///
/// Its own entry point rather than a `RemovalKind` argument at the call site, for the reason the
/// other two are: a caller that picks the kind is a caller that can pick it wrong, and a port
/// reported as an extra spends the resource budget and names the wrong `[guard]` key in the
/// refusal.
pub async fn enforce_ports(
    config: &Config,
    registry: &Arc<BackendRegistry>,
    removals: &[(String, String)],
    reaping: &Reaping,
    scope: GuardScope,
) -> Result<Reaped> {
    enforce_kind(
        config,
        registry,
        removals,
        RemovalKind::Port,
        reaping,
        scope,
    )
    .await
}

/// Both halves of the guard, which differ only in which ceiling they answer to and in what the
/// refusal tells you to do.
///
/// **The count comes off the command's [`Reaping`] and goes back onto it here**, which is what
/// makes the ceiling a budget for the command. Two entry points wrap this rather than one taking
/// a `RemovalKind`, because a caller choosing the kind is a caller who can choose it wrong â€” a
/// package teardown reported as an extra escapes `protection_of`'s declarability test.
async fn enforce_kind(
    config: &Config,
    registry: &Arc<BackendRegistry>,
    removals: &[(String, String)],
    kind: RemovalKind,
    reaping: &Reaping,
    scope: GuardScope,
) -> Result<Reaped> {
    vet(config, registry, removals, kind, reaping, scope).await?;
    // Recorded only once the set is cleared: a refused command stops here, and a removal
    // that was never allowed must not raise the total anything behind it is measured
    // against.
    reaping.record(kind, removals.len());
    Ok(Reaped {
        scope,
        allowed: removals.len(),
    })
}

/// The guard's question, asked without spending: refuse or permit, record nothing.
///
/// [`enforce_kind`]'s decision with the ledger write left out. It exists because a command
/// may ask before its confirmation prompt while the engine asks again over the same pairs
/// before carrying them out (`remove-orphans`, `purge-undeclared`). Two asks, one rule â€” and
/// one spend: the ask that merely decides must not raise `so_far`, or the engine's ask
/// measures `N + N` against the ceiling and refuses a set the user already confirmed.
pub async fn vet(
    config: &Config,
    registry: &Arc<BackendRegistry>,
    removals: &[(String, String)],
    kind: RemovalKind,
    reaping: &Reaping,
    scope: GuardScope,
) -> Result<()> {
    let mut cache = EssentialsCache::default();
    let mut report = inspect_removals(config, registry, removals, kind, reaping, &mut cache).await;

    allow_the_count(config, &mut report, scope);

    if report.is_empty() {
        return Ok(());
    }
    refuse(report.message(scope, kind))
}

/// Turn a refusal into the error every command reports.
///
/// **Every guard entry point comes through here**, so `Error::Refused` â€” U21's exit code 3 â€” is
/// a property of the guard rather than of each caller remembering to pick the right variant.
/// The install ceiling returned `Error::Other` until this existed, which made the one refusal
/// in II.10 that is about installs exit 1 while its eight siblings exited 3.
///
/// It does **not** fire `on_guard_refusal`. Announcing a refusal is a side effect, and a side
/// effect inside a decision function runs wherever the decision is evaluated â€” including in
/// tests, which call this with a default `Config` whose `config_root()` is the developer's own
/// `~/.config/shall`. That would have `cargo test` executing the developer's real hooks. The
/// event is fired once, where `Error::Refused` becomes an exit code (`finish`), which is the
/// layer where effects belong.
///
/// **Note what this function is and is not.** It is where every *guard* refusal is built. It is
/// not where every refusal in the program is built â€” the SEC/T series constructs its own, and
/// for nine sites those were `Error::Validation`, so they exited 1 and the hook never heard
/// them. What makes the promise true is the variant, not this function, and what checks it is
/// `tests/e2e/grader_refusal_exit_code_tests.rs`.
fn refuse<T>(message: String) -> Result<T> {
    Err(Error::Refused(message))
}

/// Inspect the *desired* state against the `[guard]` install rules (II.10) that do not need
/// runtime state: `deny_packages` and `pinned_only`. The two that do â€” `require_snapshot`
/// and `deny_vulnerable` â€” are enforced by the caller, which holds the snapshot provider and
/// the audit report. Returns one objection per offending package; an empty vec means the
/// spec-level rules pass.
pub fn inspect_desired(
    guard: &crate::config::GuardSettings,
    desired: &std::collections::HashMap<String, Vec<crate::core::PackageSpec>>,
) -> Vec<Objection> {
    let mut objections = Vec::new();
    for specs in desired.values() {
        for s in specs {
            let key = format!("{}:{}", s.backend, s.name);
            if guard
                .deny_packages
                .iter()
                .any(|d| d.eq_ignore_ascii_case(&s.name))
            {
                objections.push(Objection::Denied { key: key.clone() });
            }
            if guard.pinned_only {
                let pinned = s
                    .options
                    .one("version")
                    .map(|v| !v.is_empty() && v != "latest" && v != "*")
                    .unwrap_or(false);
                if !pinned {
                    objections.push(Objection::Unpinned { key });
                }
            }
        }
    }
    objections
}

/// A one-line, human-readable reason for an install-side objection, for the caller's
/// violation list. (Removal objections render through [`GuardReport::message`] instead.)
pub fn describe_objection(o: &Objection) -> String {
    match o {
        Objection::Denied { key } => format!("{} â€” denied by policy (deny_packages)", key),
        Objection::Unpinned { key } => {
            format!("{} â€” pinned_only requires an explicit @version=", key)
        }
        Objection::Protected { key, reason } => format!("{} â€” {}", key, reason),
        Objection::TooMany {
            count,
            limit,
            setting,
        } => format!("removes {} items, over {} ({})", count, setting, limit),
        Objection::TooManyInstalls { count, limit } => {
            format!("installs {} packages, over max_installs ({})", count, limit)
        }
        Objection::UnverifiedEssentials { key, backend } => format!(
            "{} â€” {} cannot currently report which packages the OS needs, so the removal \
             cannot be checked against OS-essentials",
            key, backend
        ),
    }
}

/// Refuse an oversized install set (II.10). The install-side twin of the count check in
/// [`enforce`]: `max_installs` catches a manifest that accidentally globs its way into tens
/// of thousands of installs. `Ok(())` means the install may proceed.
///
/// The override is `config.allow_mass_install` (`--allow-mass-install`), never `--yes` â€”
/// the same rule the removal ceiling follows, and for the same reason: `-y` is what every
/// script passes.
///
/// Unlike removals, installs have no protection or OS-essential dimension â€” nothing is
/// *installed* that the system forbids here â€” so the only question is the count, and `0`
/// (unset) disables it.
pub async fn enforce_installs(
    config: &Config,
    count: usize,
    reaping: &Reaping,
    scope: GuardScope,
) -> Result<()> {
    // The total counts installs, so this gate answers `max_total_changes` before its own
    // ceiling: a run that is inside `max_installs` and past the total is still past the total.
    enforce_total(config, count, reaping, scope, "install")?;
    if config.guard.max_installs == 0 || count <= config.guard.max_installs {
        reaping.record_addition(count);
        return Ok(());
    }
    if config.allow_mass_install {
        warn!(
            "the install count for '{}' ({}) was allowed by --allow-mass-install.",
            scope.as_str(),
            count
        );
        reaping.record_addition(count);
        return Ok(());
    }

    refuse(format!(
        "{}: refusing this install.\n  \
         - it installs {} packages, over the limit of {} (config: max_installs)\n\n\
         This usually means a manifest matched more than you meant â€” run `shall plan` and \
         read the counts before proceeding.\n\n\
         What to do:\n  \
         shall plan                     see exactly what would be installed\n  \
         {} --allow-mass-install carry out this install anyway",
        scope.as_str(),
        count,
        config.guard.max_installs,
        scope.as_str(),
    ))
}

/// The gate for changes that take nothing away â€” a resource created or rewritten, a port opened.
///
/// They answer to no ceiling of their own: `max_total_changes` is the only number that counts
/// them, and nothing here can be `protected` (a thing that does not exist yet cannot be a thing
/// you asked Shall to keep). It exists so the total is a total. Before it, a sync could install
/// forty packages, write forty links and open forty ports under a `max_total_changes` of ten,
/// because the only gates on the way counted removals.
pub async fn enforce_additions(
    config: &Config,
    count: usize,
    reaping: &Reaping,
    scope: GuardScope,
) -> Result<()> {
    if count == 0 {
        return Ok(());
    }
    enforce_total(config, count, reaping, scope, "change")?;
    reaping.record_addition(count);
    Ok(())
}

/// `max_total_changes`, for the gates that have no per-kind ceiling of their own.
///
/// The removal gates check it inside [`inspect_removals`] instead, where it joins the rest of
/// the report and gets rendered with the protections and the per-kind count in one refusal.
///
/// **`pub` so the guard's tests can reach it.** They live in `tests/app/sync/guard/`, which
/// links the library without `cfg(test)`, and a private item is invisible from there.
pub fn enforce_total(
    config: &Config,
    count: usize,
    reaping: &Reaping,
    scope: GuardScope,
    noun: &str,
) -> Result<()> {
    let Some(Objection::TooMany {
        count,
        limit,
        setting,
    }) = too_many_changes(config, reaping, count)
    else {
        return Ok(());
    };
    // Either flag answers it, because both say the same sentence â€” "yes, that many, I meant it"
    // (`Y20`) â€” and a total is made of removals and installs both. A third flag for the third
    // ceiling would be a third way to say one thing.
    if config.allow_mass_removal || config.allow_mass_install {
        if let Some(said) = announcement(
            &[Objection::TooMany {
                count,
                limit,
                setting,
            }],
            config,
            scope,
        ) {
            warn!("{}", said);
        }
        return Ok(());
    }
    refuse(format!(
        "{}: refusing this {}.\n  \
         - it makes {} changes in total, over the limit of {} ([guard] {})\n\n\
         This ceiling counts everything one command does â€” installs and upgrades, packages \
         removed, resources torn down or written, ports opened and closed. The per-kind limits \
         each passed; the total did not.\n\n\
         What to do:\n  \
         shall plan                     see exactly what would change\n  \
         [guard] {}       raise or clear the total (preferences.toml)\n  \
         <command> --allow-mass-removal carry out this run anyway\n  \
         <command> --allow-mass-install the same â€” this total answers to either flag",
        scope.as_str(),
        noun,
        count,
        limit,
        TOTAL_KEY,
        TOTAL_KEY,
    ))
}

/// **A transaction undoing its own work asks here** (owner ruling, 2026-09-28).
///
/// The guard, for real: every pair is checked against `protected_packages` and the OS-essential
/// list, and a refusal is a refusal. What it does *not* do is consult the ceilings or write to the
/// ledger, because **undoing what this run installed thirty seconds ago is not a removal of the
/// user's software** — and a compensation that could be refused for exceeding `max_removals`
/// strands a machine half-unwound, which is the one state no part of this is trying to produce.
///
/// **All or nothing, and that is a choice.** One protected package in the set refuses the whole
/// unwind, so the machine keeps everything the transaction installed and the caller says so. A
/// per-pair answer would leave the machine holding one package the manifest asks for and not the
/// nine it asked to undo — a state neither the user nor the manifest describes, and one nobody
/// would believe a rollback had produced. The refusal names the package, so "we could not fully
/// unwind" is answerable.
pub async fn enforce_own_work(
    config: &Config,
    registry: &Arc<BackendRegistry>,
    removals: &[(String, String)],
) -> Result<Reaped> {
    let mut cache = EssentialsCache::default();
    let mut report = inspect_removals(
        config,
        registry,
        removals,
        RemovalKind::Package,
        // A ledger of its own, so a rollback's ask is measured against nothing. The ceilings are
        // dropped below anyway; this is here so `inspect_removals` reads the same shape it always
        // has rather than a special case threaded through it.
        &Reaping::default(),
        &mut cache,
    )
    .await;
    // **The ceilings are this function's one difference, and it is a difference in what is
    // asked, not in what is checked.** Protection and OS-essentials are not "are you sure" — they
    // are the reason the guard exists — and a rollback that deleted a package the user said never
    // to remove would be the guard's exact failure with a sympathetic story attached.
    report
        .objections
        .retain(|o| !matches!(o, Objection::TooMany { .. }));
    if !report.is_empty() {
        return Err(refusal_for_own_work(report, removals));
    }
    Ok(Reaped {
        scope: GuardScope::Sync,
        allowed: removals.len(),
    })
}

/// A rollback's refusal, in the shape the transaction needs: the package that is protected, and
/// the sentence that says the machine kept what Shall installed.
fn refusal_for_own_work(report: GuardReport, removals: &[(String, String)]) -> Error {
    let protected: Vec<String> = report
        .objections
        .iter()
        .filter_map(|o| match o {
            Objection::Protected { key, reason } => Some(format!("{key} ({reason})")),
            Objection::UnverifiedEssentials { key, backend } => Some(format!(
                "{key}: {backend} could not be asked what it considers essential, so Shall is not \
                 removing anything it owns"
            )),
            _ => None,
        })
        .collect();
    Error::Refused(format!(
        "not undoing this run's work: {}.\n  \
         Shall installed these seconds ago as part of a transaction that then failed, and \
         removing them again is exactly what the guard is for — but the ceilings that bound a \
         real removal do not apply to undoing Shall's own work, and protection does.\n  \
         What is on the machine now: {} package(s) this run installed, left in place and \
         recorded as Shall's.",
        if protected.is_empty() {
            "the guard declined".to_string()
        } else {
            protected.join("; ")
        },
        removals.len()
    ))
}

/// Charge a batch of mutations the model cannot name â€” `@undo=` shell commands above all â€”
/// against the command's total, then record them.
///
/// These are the one mutation family with no per-kind ceiling to answer, because nothing can
/// inspect a shell command for what it will remove. That made them invisible to
/// `max_total_changes` entirely: a config departing forty `exec:` lines ran forty arbitrary
/// teardowns past every gate. The count is the only fact the guard can hold about them, and it
/// is enough for the ceiling that exists to catch exactly this shape.
///
/// **`pub` so the guard's tests can reach it.** They live in `tests/app/sync/guard/`, which
/// links the library without `cfg(test)`, and a private item is invisible from there.
pub fn charge_unmodelled(
    config: &Config,
    reaping: &Reaping,
    count: usize,
    scope: GuardScope,
) -> Result<()> {
    enforce_total(config, count, reaping, scope, "batch of exec undos")?;
    reaping.record_addition(count);
    Ok(())
}

/// Enforce for `purge-undeclared`, where the count is not the question (II.11).
///
/// `max_removals` catches accidents, and this command is the opposite of an accident: you
/// typed its name and confirmed it. **`protected_packages` and OS-essential still apply** â€”
/// those are not "are you sure", and the ratio check (II.11) is what asks whether you meant
/// it at all.
pub async fn enforce_deliberate(
    config: &Config,
    registry: &Arc<BackendRegistry>,
    removals: &[(String, String)],
    reaping: &Reaping,
    scope: GuardScope,
) -> Result<Reaped> {
    vet_deliberate(config, registry, removals, scope).await?;
    // Still recorded. The count is not the question *for this command*, but a purge is one
    // phase of a run that goes on to tear extras down, and the budget those answer to has
    // to know what has already gone.
    reaping.record(RemovalKind::Package, removals.len());
    Ok(Reaped {
        scope,
        allowed: removals.len(),
    })
}

/// [`enforce_deliberate`]'s decision with the ledger write left out â€” [`vet`]'s twin for the
/// deliberate scopes, so `purge-undeclared`'s prompt-time ask does not spend against the total
/// its engine ask is about to measure.
pub async fn vet_deliberate(
    config: &Config,
    registry: &Arc<BackendRegistry>,
    removals: &[(String, String)],
    scope: GuardScope,
) -> Result<()> {
    let mut report = inspect(config, registry, removals).await;
    report
        .objections
        .retain(|o| !matches!(o, Objection::TooMany { .. }));
    if report.is_empty() {
        return Ok(());
    }
    refuse(report.message(scope, RemovalKind::Package))
}

/// Split an extras-ledger key (`link:/home/u/.vimrc`, `repo:apt:ppa:x/y`) into the
/// `(kind, id)` pair the guard inspects.
///
/// **A key this build cannot parse is carried through under an empty kind rather than dropped.**
/// The guard must never silently stop covering something it could not read: an unreadable row
/// still names a thing on the machine, and the count and the protection rules both still apply
/// to it. Only the *kind* is unknown, and the guard does not dispatch on kind.
pub fn extra_removal_pairs(keys: &[String]) -> Vec<(String, String)> {
    use crate::core::extras_lock::ExtraKey;
    keys.iter()
        .map(|k| match k.parse::<ExtraKey>() {
            Ok(key) => (key.kind.to_string(), key.subject),
            Err(()) => (String::new(), k.clone()),
        })
        .collect()
}

/// Pull the `(backend, name)` removal pairs out of a planned change set.
pub fn removal_pairs(changes: &super::planner::SyncChanges) -> Vec<(String, String)> {
    use crate::core::GraphAction;
    changes
        .graph
        .node_weights()
        .filter_map(|w| match w {
            GraphAction::Remove { name, backend } => Some((backend.clone(), name.clone())),
            _ => None,
        })
        .collect()
}
