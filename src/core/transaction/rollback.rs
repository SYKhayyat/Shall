//! What a failed transaction undoes.
//!
//! **Two questions, in two functions.** *What* to put back is a pure answer about the graph, the
//! history and the machine's protection rules, and it is decided by [`plan_undo`], which is
//! unit-testable without a registry, a journal or a running manager. *Whether putting it back
//! worked* is [`Transaction::apply_undo`]'s, and it is the half that talks to managers.
//!
//! This was one 290-line method, and the state it hid is the state this engine exists to keep
//! straight: which installs have a version to go back to, which have none and must be removed,
//! which removals must be re-instated, and which of all of those the guard will not allow.

use super::*;

/// One compensating action, decided before any of them is taken.
pub enum Undo {
    /// This run upgraded something that was already here. Put the old version back — removing
    /// it would uninstall software the user had, which is the opposite of a rollback.
    RestoreVersion {
        backend: String,
        name: String,
        version: String,
    },
    /// This run installed something that was not here and nothing declares. Take it off.
    RemoveInstalled { backend: String, name: String },
    /// This run removed something that was here and something still wants. Put it back.
    ReinstateRemoved {
        backend: String,
        name: String,
        version: Option<String>,
    },
}

impl Undo {
    pub fn backend(&self) -> &str {
        match self {
            Undo::RestoreVersion { backend, .. }
            | Undo::RemoveInstalled { backend, .. }
            | Undo::ReinstateRemoved { backend, .. } => backend,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Undo::RestoreVersion { name, .. }
            | Undo::RemoveInstalled { name, .. }
            | Undo::ReinstateRemoved { name, .. } => name,
        }
    }
}

/// What a rollback will do, and what it refuses to do, decided before it does either.
pub struct RollbackPlan {
    /// The compensations, in the order the work that needs undoing finished — walked in reverse,
    /// so the machine comes back the way it went.
    pub steps: Vec<Undo>,
    /// Compensations the plan will not make, each a sentence the user is shown. These are not
    /// errors: they are the cases where undoing would destroy something nobody asked to change.
    pub failures: Vec<String>,
}

/// The facts a rollback plan is decided from.
///
/// Handed in rather than read off a `Transaction`, so the planner can be exercised on a graph and
/// a history with no registry, journal, config file or manager behind them — which is the only
/// way the four refusal branches below can each be put under load.
pub struct UndoFacts<'a> {
    /// Does this plan intend the machine to end up holding this package? `None` when the run is
    /// not reconciling against a manifest and the question has no answer (`U41`).
    pub intends_present: &'a dyn Fn(&str, &str) -> Option<bool>,
    /// `backend:name` pairs the running OS reports as essential.
    pub essential: &'a HashSet<String>,
    /// Backends whose essential question has no answer this run. A removal through one of these
    /// cannot be checked against what the OS needs, so it is refused rather than read as silence.
    pub unanswered: &'a std::collections::BTreeSet<String>,
    /// The user's configuration, for `unprotected_packages` and the escape hatch.
    pub config: &'a crate::config::Config,
    /// Whether this backend exists here and can install — the two questions that have to be yes
    /// before a removal is possible at all.
    pub removable: &'a dyn Fn(&str) -> bool,
}

/// **What to undo, in the order it has to be undone.**
///
/// Pure: no journal, no manager, no process. The two arms are the same rule seen from both sides
/// (`U41`): *rollback does not undo work that moved the machine toward the declared state.*
pub fn plan_undo(
    history: &[(NodeIndex, Prior)],
    graph: &StableDiGraph<GraphAction, ()>,
    facts: &UndoFacts<'_>,
) -> RollbackPlan {
    let mut steps: Vec<Undo> = Vec::new();
    let mut failures: Vec<String> = Vec::new();

    for (idx, prior) in history.iter().rev() {
        match &graph[*idx] {
            GraphAction::Install(spec) => match prior {
                Prior::Present(version) => {
                    // It was already there. Undoing an upgrade is putting the old version back —
                    // removing the package uninstalls software the user had, which is the
                    // opposite of a rollback.
                    let Some(version) = version.clone() else {
                        warn!(
                            "rollback cannot revert {}:{}: its manager did not report a version \
                             before the change, so there is none to go back to. It stays at the \
                             version this run installed.",
                            spec.backend, spec.name
                        );
                        failures.push(format!(
                            "{}:{} (left at the new version)",
                            spec.backend, spec.name
                        ));
                        continue;
                    };
                    steps.push(Undo::RestoreVersion {
                        backend: spec.backend.clone(),
                        name: spec.name.clone(),
                        version,
                    });
                }
                Prior::Absent => {
                    // **`Prior::Absent` is not permission to remove.** It says the package was not
                    // here before this run; it does not say nobody wants it. If the manifest still
                    // declares it, this install is the goal reached early, and compensating it
                    // hands the next `sync` the same work to do again — the transaction's own
                    // comment claims rollback "puts this back", and removing something nothing
                    // asked it to remove is the opposite.
                    if (facts.intends_present)(&spec.backend, &spec.name) == Some(true) {
                        info!(
                            "rollback is leaving {}:{} installed — it succeeded and the manifest \
                             still declares it, so removing it would only give the next sync the \
                             same work to do again.",
                            spec.backend, spec.name
                        );
                        continue;
                    }
                    if facts.unanswered.contains(&spec.backend) {
                        // The manager cannot say what the OS needs right now, so the protection
                        // question cannot be answered. Rollback reports "incomplete" anyway when
                        // it cannot finish; leaving software installed is its safe direction,
                        // never removing blind.
                        error!(
                            "rollback will not remove {}:{} — `{}` cannot currently report which \
                             packages the OS needs, so the protection check is unavailable. It \
                             stays installed.",
                            spec.backend, spec.name, spec.backend
                        );
                        failures.push(format!(
                            "{}:{} (essential check unavailable, left installed)",
                            spec.backend, spec.name
                        ));
                        continue;
                    }
                    if let Some(p) = crate::app::sync::guard::protection_of(
                        facts.config,
                        Some(&spec.backend),
                        &spec.name,
                        facts.essential,
                    ) {
                        error!(
                            "rollback will not remove {}:{} — {}. It stays installed, and this \
                             transaction is left partly applied.",
                            spec.backend,
                            spec.name,
                            p.reason()
                        );
                        failures.push(format!(
                            "{}:{} (protected, left installed)",
                            spec.backend, spec.name
                        ));
                        continue;
                    }
                    if !(facts.removable)(&spec.backend) {
                        continue;
                    }
                    // `protection_of` above has already said whether this pair may be removed,
                    // and the guard is asked once for the whole set when it is applied, so a
                    // rollback is permitted by the guard rather than by a token it made itself.
                    // The per-package message up there is what names the install that was kept,
                    // which is why both are here.
                    steps.push(Undo::RemoveInstalled {
                        backend: spec.backend.clone(),
                        name: spec.name.clone(),
                    });
                }
                // Not knowing whether the user already had it is not permission to delete it.
                Prior::Unknown => {
                    warn!(
                        "rollback will not remove {}:{}: Shall could not tell whether it was \
                         already installed before this run, and removing what you may have had \
                         is not something you asked for.",
                        spec.backend, spec.name
                    );
                    failures.push(format!(
                        "{}:{} (left installed — prior state unknown)",
                        spec.backend, spec.name
                    ));
                }
            },
            GraphAction::Remove { name, backend } => {
                // Nothing was there to lose.
                if prior == &Prior::Absent {
                    continue;
                }
                // **The install arm's rule, from the other side** (`U41`). This removal happened
                // because nothing in the plan intends the package to be present, and that fact
                // is still true — it is the same set that authorised the removal, asked the same
                // way. Re-installing it would hand the next sync the same work to do again.
                //
                // `declared` is `None` for the runs where a removal is not a reconciliation — a
                // `rebuild`'s down phase, a hand-typed `uninstall` — and there the reinstate
                // below is exactly right.
                if (facts.intends_present)(backend, name) == Some(false) {
                    info!(
                        "rollback is leaving {}:{} removed — nothing declares it, so putting it \
                         back would only give the next sync the same work to do again. `shall \
                         history` and the pre-sync snapshot are how it comes back.",
                        backend, name
                    );
                    continue;
                }
                let version = match prior {
                    Prior::Present(v) => v.clone(),
                    _ => None,
                };
                steps.push(Undo::ReinstateRemoved {
                    backend: backend.clone(),
                    name: name.clone(),
                    version,
                });
            }
        }
    }

    RollbackPlan { steps, failures }
}

impl Transaction {
    pub(super) async fn rollback(&mut self) -> Result<()> {
        debug!("reverting modification history");
        // A compensating action that itself fails leaves the system in a partial state — most
        // dangerously, a package the user HAD, that this transaction removed, and that the
        // reinstall could not bring back. Swallowing that error is the worst place in the
        // codebase to be quiet (H2): the user is told the transaction failed and rolled back,
        // while a package is silently gone. Report every failure by name, and return Err so the
        // caller can say the rollback was incomplete.
        let history = self.history.clone();
        // Recovery paths are removal paths, and they need the guard more than ordinary ones
        // because nobody is watching (V.64). These removals are issued at execution time and
        // never pass the plan-time gate in `sync`, so this is the only place they can be
        // checked.
        let backends: HashSet<String> = history
            .iter()
            .filter_map(|(idx, _)| match &self.graph[*idx] {
                GraphAction::Install(s) => Some(s.backend.clone()),
                GraphAction::Remove { .. } => None,
            })
            .collect();
        let answers = crate::app::sync::guard::essential_names(
            &self.registry,
            &backends,
            self.config.max_concurrent,
            &mut Default::default(),
        )
        .await;

        let plan = {
            let facts = UndoFacts {
                intends_present: &|b, n| self.plan_intends_present(b, n),
                essential: &answers.names,
                unanswered: &answers.unanswered,
                config: &self.app_config,
                removable: &|b| {
                    self.registry
                        .get(b)
                        .is_some_and(|cap| cap.as_installable().is_some())
                },
            };
            plan_undo(&history, &self.graph, &facts)
        };

        self.apply_undo(plan).await
    }

    /// Carry out a plan: put versions back, re-instate what this run removed, and take off what
    /// it installed — the last through the removal guard, asked once for the whole set.
    async fn apply_undo(&self, plan: RollbackPlan) -> Result<()> {
        let mut failures = plan.failures;
        // **The installs this rollback will undo, gathered and then removed in one guarded
        // pass.** It used to mint a token per package — `Reaped::for_reason` with a reason the
        // type threw away — so "the guard ran" was true of nothing at all. The guard is asked
        // below, for the whole set, and the token it returns says how many it cleared.
        let mut to_undo: Vec<(String, String)> = Vec::new();

        for step in plan.steps {
            match step {
                Undo::RestoreVersion {
                    backend,
                    name,
                    version,
                } => {
                    if let Err(e) = self
                        .reinstate(&backend, &name, &Some(version.clone()))
                        .await
                    {
                        error!(
                            "rollback could not put {}:{} back to {}: {}",
                            backend, name, version, e
                        );
                        failures.push(format!("{}:{} (left at the new version)", backend, name));
                    }
                }
                Undo::ReinstateRemoved {
                    backend,
                    name,
                    version,
                } => {
                    if let Err(e) = self.reinstate(&backend, &name, &version).await {
                        error!(
                            "rollback could not reinstall {}:{} that this run removed — it is now \
                             MISSING: {}",
                            backend, name, e
                        );
                        failures.push(format!("{}:{} (now missing)", backend, name));
                    }
                }
                Undo::RemoveInstalled { backend, name } => to_undo.push((backend, name)),
            }
        }

        // **The guard, asked once, for everything this run installed and is now undoing.** It is
        // a real ask with a real answer — `protected_packages` and the OS-essential list are
        // consulted, and the token that comes back carries the number of packages it cleared —
        // and it deliberately does not consult the ceilings, because undoing Shall's own work is
        // not a removal of the user's software, and a compensation that could be refused for
        // exceeding `max_removals` would strand the machine half-changed.
        if !to_undo.is_empty() {
            let pairs = to_undo.clone();
            match crate::app::sync::guard::enforce_own_work(
                &self.app_config,
                &self.registry,
                &pairs,
            )
            .await
            {
                Ok(reaped) => {
                    debug_assert!(
                        reaped.authorises(pairs.len()),
                        "the guard cleared {} of the {} pairs it was asked about",
                        reaped.allowed(),
                        pairs.len()
                    );
                    for (backend, name) in &pairs {
                        let Some(b) = self.registry.get(backend) else {
                            continue;
                        };
                        let Some(h) = b.as_installable() else {
                            continue;
                        };
                        if let Err(e) = h
                            .remove(std::slice::from_ref(name), b.sudo_for_write(), reaped)
                            .await
                        {
                            error!(
                                "rollback could not remove {}:{} that this run installed — it \
                                 remains on the system: {}",
                                backend, name, e
                            );
                            failures.push(format!("{}:{} (left installed)", backend, name));
                        }
                    }
                }
                // The guard would not undo it. The machine keeps what the transaction installed,
                // and says so — the alternative is removing software the user said never to
                // remove because Shall put it there a minute ago.
                Err(e) => {
                    error!(
                        "rollback is not undoing {} package(s) this run installed: {}",
                        pairs.len(),
                        e
                    );
                    for (backend, name) in &pairs {
                        failures.push(format!(
                            "{}:{} (left installed — the guard declined)",
                            backend, name
                        ));
                    }
                }
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(Error::Transaction(format!(
                "rollback was incomplete — {} compensating action(s) failed: {}",
                failures.len(),
                failures.join(", ")
            )))
        }
    }

    /// Put one package back the way it was, at the version it was at.
    async fn reinstate(&self, backend: &str, name: &str, version: &Option<String>) -> Result<()> {
        let Some(b) = self.registry.get(backend) else {
            return Err(Error::BackendNotFound(backend.to_string()));
        };
        let Some(h) = b.as_installable() else {
            return Err(Error::Transaction(format!(
                "backend `{}` cannot install",
                backend
            )));
        };
        let mut options = crate::config::grammar::Options::default();
        if let Some(v) = version {
            // Without this the reinstall takes whatever is newest, so a rolled-back removal
            // silently loses its pin — the package comes back at a version nobody declared.
            options.set("version", v.clone());
        }
        h.install(
            &[PackageSpec {
                name: name.to_string(),
                backend: backend.to_string(),
                options,
                requires: vec![],
                present: true,
            }],
            b.sudo_for_write(),
        )
        .await
        .map(|_| ())
    }
}
