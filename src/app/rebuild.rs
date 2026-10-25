//! `shall rebuild` — assert the declared set from scratch (X.1).
//!
//! Convergence is blind to state that is wrong while the difference is empty: a half-configured
//! package, an interrupted extraction, a closure whose dependencies were removed by hand. In
//! every such case the machine and the declaration agree at the level `sync` inspects, so no
//! amount of re-running it changes anything. Rebuild stops asking what changed and removes the
//! declared packages so it can install them again.
//!
//! Everything here is pure: scope selection and batch ordering take facts and return a plan.
//! The applying half lives in the caller, which holds the registry and the sync engine.

use crate::core::PackageSpec;

/// What to rebuild. A bare `rebuild` defaults to `All` (K2, ruled 2026-07-24) — but the CLI
/// warns loudly before it does, because the failure mode is software missing from a machine and
/// `--all` is a large thing to reach by pressing enter. The warning is the safeguard the old
/// "refuse and list the forms" design used a refusal for; the owner chose warn-and-proceed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    Packages(Vec<Target>),
    Backend(String),
    All,
}

/// A `shall rebuild NAME` argument, after the one parser has had it.
///
/// Split where the backend registry can be consulted rather than at the point of use: a
/// `next_back()` on `:` turns `web:https://host/x.tar.gz` into `//host/x.tar.gz`, and never
/// checks that the prefix names a backend at all (C13).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub backend: Option<String>,
    pub name: String,
    /// What the user typed, for the message that names it back to them.
    pub raw: String,
}

impl Target {
    pub fn parse(input: &str, is_known_backend: impl Fn(&str) -> bool) -> Self {
        let (backend, name) = crate::config::parser::split_removal_target(input, is_known_backend);
        Self {
            backend,
            name,
            raw: input.to_string(),
        }
    }

    fn matches(&self, spec: &PackageSpec) -> bool {
        self.name == spec.name && self.backend.as_ref().is_none_or(|b| b == &spec.backend)
    }
}

/// A package left out of the rebuild, and the sentence explaining it. Printed, never silent:
/// a rebuild that quietly skipped half its scope would report success over an unrepaired
/// machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    pub key: String,
    pub reason: String,
}

/// One backend's packages: all of them come down, then all of them go back up (K1).
#[derive(Debug, Clone)]
pub struct Batch {
    pub backend: String,
    pub specs: Vec<PackageSpec>,
}

impl Batch {
    pub fn names(&self) -> Vec<String> {
        self.specs.iter().map(|s| s.name.clone()).collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Plan {
    pub batches: Vec<Batch>,
    pub skipped: Vec<Skipped>,
}

impl Plan {
    pub fn is_empty(&self) -> bool {
        self.batches.iter().all(|b| b.specs.is_empty())
    }

    pub fn total(&self) -> usize {
        self.batches.iter().map(|b| b.specs.len()).sum()
    }
}

/// Whether this backend owns the shell and the system libraries.
///
/// `needs_root()` already draws exactly this line — a manager that must be root to install is
/// one installing into the system, and one that need not is installing into a home directory.
/// A second list of "system backends" would be a second answer to a question already answered.
pub type IsFoundation<'a> = &'a dyn Fn(&str) -> bool;

/// Foundation backends first, then the rest, each tier keeping `priority` order (II.6).
///
/// Dependency direction is what fixes this, not blast radius: a `cargo` crate can need a
/// system compiler, and no `apt` package has ever needed a crate. Rebuilding user-space
/// software first would rebuild it against the system state the rebuild is about to replace,
/// leaving it stale the moment the foundation batch lands.
pub fn order_backends(
    backends: &[String],
    priority: &[String],
    is_foundation: IsFoundation,
) -> Vec<String> {
    let rank = |b: &String| priority.iter().position(|p| p == b).unwrap_or(usize::MAX);

    let mut ordered: Vec<String> = backends.to_vec();
    ordered.sort_by_key(|b| (!is_foundation(b), rank(b), b.clone()));
    ordered.dedup();
    ordered
}

/// Which declared packages the scope selects, batched by backend and ordered.
///
/// `declared` is the resolved desired state; `installed` answers whether Shall currently
/// manages that package. A declared package that is not installed is left to `sync` — putting
/// it in a rebuild would mean removing something that is not there in order to install it,
/// which is an install, and `sync` is the command for installs.
pub fn plan(
    scope: &Scope,
    declared: &[PackageSpec],
    installed: &dyn Fn(&str, &str) -> bool,
    priority: &[String],
    is_foundation: IsFoundation,
) -> Plan {
    let mut plan = Plan::default();

    let selected: Vec<&PackageSpec> = declared
        .iter()
        .filter(|s| s.present)
        .filter(|s| match scope {
            Scope::All => true,
            Scope::Backend(b) => &s.backend == b,
            Scope::Packages(targets) => targets.iter().any(|t| t.matches(s)),
        })
        .collect();

    // A named package that is not declared is the user believing something is managed when it
    // is not. Rebuild must say so rather than silently rebuild nothing.
    if let Scope::Packages(targets) = scope {
        for target in targets {
            if !selected.iter().any(|s| target.matches(s)) {
                plan.skipped.push(Skipped {
                    key: target.raw.clone(),
                    reason: "not declared in any active module".to_string(),
                });
            }
        }
    }

    let mut by_backend: std::collections::HashMap<String, Vec<PackageSpec>> = Default::default();
    for spec in selected {
        if !installed(&spec.backend, &spec.name) {
            plan.skipped.push(Skipped {
                key: format!("{}:{}", spec.backend, spec.name),
                reason: "declared but not installed — `sync` installs it".to_string(),
            });
            continue;
        }
        by_backend
            .entry(spec.backend.clone())
            .or_default()
            .push(spec.clone());
    }

    let names: Vec<String> = by_backend.keys().cloned().collect();
    for backend in order_backends(&names, priority, is_foundation) {
        let mut specs = by_backend.remove(&backend).unwrap_or_default();
        specs.sort_by(|a, b| a.name.cmp(&b.name));
        plan.batches.push(Batch { backend, specs });
    }

    plan.skipped.sort_by(|a, b| a.key.cmp(&b.key));
    plan
}

/// Drop packages the guard would refuse to remove, recording each.
///
/// Rebuild removes in order to reinstall, but the guard cannot know that, and it is right not
/// to trust it: if the reinstall fails, the machine is genuinely without the package. Narrowing
/// the scope here keeps `rebuild --all` usable on a machine whose `bash` is protected, without
/// weakening the refusal or asking the guard to make an exception for one caller.
pub fn without_protected(plan: &mut Plan, protection: &dyn Fn(&str, &str) -> Option<String>) {
    for batch in &mut plan.batches {
        batch
            .specs
            .retain(|spec| match protection(&batch.backend, &spec.name) {
                Some(reason) => {
                    plan.skipped.push(Skipped {
                        key: format!("{}:{}", batch.backend, spec.name),
                        reason,
                    });
                    false
                }
                None => true,
            });
    }
    plan.batches.retain(|b| !b.specs.is_empty());
    plan.skipped.sort_by(|a, b| a.key.cmp(&b.key));
}
