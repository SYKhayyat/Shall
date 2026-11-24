//! **What a failed run undoes is a pure decision, and it is tested without a machine.**
//!
//! `rollback` used to be one 290-line method that decided and acted in the same pass, so the
//! only way to exercise a refusal was to run a real transaction against a mock backend and read
//! the log afterwards. The decision is now `plan_undo` — a function over a graph, a history and
//! the machine's protection rules — and each of its branches is asserted here directly.
//!
//! Every arm below is a rule with a story behind it, and the stories are the reason they are
//! worth pinning rather than the arithmetic:
//!
//! * **An upgrade is undone at the old version, not by removal.** Removing what the user already
//!   had is the opposite of a rollback.
//! * **`Prior::Absent` is not permission to remove** (`U41`). If the manifest still declares it,
//!   the install is the goal reached early; undoing it hands the next `sync` the same work.
//! * **`Prior::Unknown` is not permission either.** Not knowing whether the user had it is not a
//!   licence to delete it.
//! * **The failure classes are distinct sentences**, because "left at the new version" and "left
//!   installed — prior state unknown" are different facts about the machine.

use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use shall::config::grammar::Options;
use shall::core::transaction::Prior;
use shall::core::transaction::{plan_undo, RollbackPlan, Undo, UndoFacts};
use shall::core::{GraphAction, PackageSpec};
use std::collections::{BTreeSet, HashSet};

fn install(name: &str, backend: &str) -> GraphAction {
    GraphAction::Install(PackageSpec {
        name: name.into(),
        backend: backend.into(),
        options: Options::default(),
        requires: vec![],
        present: true,
    })
}

fn remove(name: &str, backend: &str) -> GraphAction {
    GraphAction::Remove {
        name: name.into(),
        backend: backend.into(),
    }
}

/// Everything a plan is decided from, with the ordinary case as the default: a manifest that
/// declares nothing, nothing essential, every backend removable.
struct Facts<'a> {
    /// **`None` is not an empty manifest.** It is a run that is not reconciling at all — a
    /// `rebuild`'s down phase, a hand-typed `uninstall` — where "does the manifest want this?"
    /// has no answer, and `U41` treats the two differently in both arms.
    declared: Option<HashSet<String>>,
    essential: HashSet<String>,
    unanswered: BTreeSet<String>,
    config: shall::config::Config,
    removable: &'a dyn Fn(&str) -> bool,
}

impl Default for Facts<'_> {
    fn default() -> Self {
        Self {
            declared: Some(HashSet::new()),
            essential: HashSet::new(),
            unanswered: BTreeSet::new(),
            config: shall::config::Config::default(),
            removable: &|_| true,
        }
    }
}

/// A plan for one run: these actions, each with the state it found before it ran.
fn plan_for(actions: &[GraphAction], priors: &[Prior], facts: &Facts<'_>) -> RollbackPlan {
    let mut graph: StableDiGraph<GraphAction, ()> = StableDiGraph::new();
    let mut history: Vec<(NodeIndex, Prior)> = Vec::new();
    for (action, prior) in actions.iter().zip(priors.iter()) {
        let idx = graph.add_node(action.clone());
        history.push((idx, prior.clone()));
    }

    let declared = facts.declared.clone();
    let intends = move |backend: &str, name: &str| {
        declared
            .as_ref()
            .map(|d| d.contains(&format!("{}:{}", backend, name)))
    };

    let f = UndoFacts {
        intends_present: &intends,
        essential: &facts.essential,
        unanswered: &facts.unanswered,
        config: &facts.config,
        removable: facts.removable,
    };
    plan_undo(&history, &graph, &f)
}

fn declared(names: &[&str]) -> HashSet<String> {
    names.iter().map(|n| n.to_string()).collect()
}

#[test]
fn an_upgrade_is_undone_at_the_old_version_rather_than_by_removal() {
    let facts = Facts {
        declared: Some(declared(&["apt:jq"])),
        ..Facts::default()
    };
    let plan = plan_for(
        &[install("jq", "apt")],
        &[Prior::Present(Some("1.6".into()))],
        &facts,
    );

    assert_eq!(plan.steps.len(), 1, "the upgrade needs exactly one undo");
    match &plan.steps[0] {
        Undo::RestoreVersion {
            backend,
            name,
            version,
        } => {
            assert_eq!((backend.as_str(), name.as_str()), ("apt", "jq"));
            assert_eq!(version, "1.6", "the version it was on, not the new one");
        }
        other => panic!(
            "an upgrade is undone by putting the old version back, not by {:?}",
            other.name()
        ),
    }
    assert!(plan.failures.is_empty());
}

#[test]
fn a_version_the_manager_never_reported_is_named_rather_than_guessed() {
    let plan = plan_for(
        &[install("jq", "apt")],
        &[Prior::Present(None)],
        &Facts::default(),
    );

    assert!(plan.steps.is_empty(), "there is no version to go back to");
    assert_eq!(plan.failures.len(), 1);
    assert!(
        plan.failures[0].contains("left at the new version"),
        "the failure has to say what was left behind: {:?}",
        plan.failures
    );
}

#[test]
fn an_install_the_manifest_still_wants_is_not_undone() {
    let facts = Facts {
        declared: Some(declared(&["apt:jq"])),
        ..Facts::default()
    };
    let plan = plan_for(&[install("jq", "apt")], &[Prior::Absent], &facts);

    assert!(
        plan.steps.is_empty(),
        "the install is the goal reached early, so undoing it is un-convergence (U41)"
    );
    assert!(
        plan.failures.is_empty(),
        "and it is not a failure — nothing was left going wrong"
    );
}

#[test]
fn an_install_nothing_declares_is_removed() {
    let facts = Facts {
        declared: Some(declared(&[])),
        ..Facts::default()
    };
    let plan = plan_for(&[install("jq", "apt")], &[Prior::Absent], &facts);

    assert_eq!(plan.steps.len(), 1);
    assert!(matches!(plan.steps[0], Undo::RemoveInstalled { .. }));
    assert!(plan.failures.is_empty());
}

#[test]
fn an_unknown_prior_state_is_not_permission_to_delete() {
    let plan = plan_for(
        &[install("jq", "apt")],
        &[Prior::Unknown],
        &Facts::default(),
    );

    assert!(plan.steps.is_empty(), "nobody knows if the user had it");
    assert_eq!(plan.failures.len(), 1);
    assert!(
        plan.failures[0].contains("prior state unknown"),
        "the sentence has to say which question was unanswerable: {:?}",
        plan.failures
    );
}

#[test]
fn an_essential_check_that_could_not_be_made_refuses_the_removal() {
    let mut facts = Facts {
        declared: Some(declared(&[])),
        ..Facts::default()
    };
    facts.unanswered.insert("apt".to_string());
    let plan = plan_for(&[install("libc6", "apt")], &[Prior::Absent], &facts);

    assert!(
        plan.steps.is_empty(),
        "a removal that cannot be checked against what the OS needs is not made"
    );
    assert_eq!(plan.failures.len(), 1);
    assert!(
        plan.failures[0].contains("essential check unavailable"),
        "{:?}",
        plan.failures
    );
}

#[test]
fn a_protected_package_is_reported_rather_than_removed() {
    let facts = Facts {
        declared: Some(declared(&[])),
        ..Facts::default()
    };
    // `sudo` is on the default `protected_packages` list on every platform, so this is the
    // ordinary rule rather than a hand-written config.
    let plan = plan_for(&[install("sudo", "apt")], &[Prior::Absent], &facts);

    assert!(plan.steps.is_empty(), "the guard refused it");
    assert_eq!(plan.failures.len(), 1);
    assert!(
        plan.failures[0].contains("protected"),
        "{:?}",
        plan.failures
    );
}

#[test]
fn an_install_the_backend_is_gone_for_is_left_alone_silently() {
    let facts = Facts {
        declared: Some(declared(&[])),
        removable: &|_| false,
        ..Facts::default()
    };
    let plan = plan_for(&[install("jq", "apt")], &[Prior::Absent], &facts);

    assert!(
        plan.steps.is_empty(),
        "there is no backend left to issue the removal"
    );
    assert!(
        plan.failures.is_empty(),
        "and nothing to tell the user: the backend's absence is not this rollback's finding"
    );
}

#[test]
fn a_removal_something_still_wants_is_put_back_at_its_version() {
    let facts = Facts {
        declared: Some(declared(&["apt:jq"])),
        ..Facts::default()
    };
    let plan = plan_for(
        &[remove("jq", "apt")],
        &[Prior::Present(Some("1.6".into()))],
        &facts,
    );

    assert_eq!(plan.steps.len(), 1);
    match &plan.steps[0] {
        Undo::ReinstateRemoved { version, .. } => {
            assert_eq!(version.as_deref(), Some("1.6"));
        }
        other => panic!("expected a reinstall, got {:?}", other.name()),
    }
}

#[test]
fn a_removal_nothing_declares_stays_removed() {
    let facts = Facts {
        declared: Some(declared(&[])),
        ..Facts::default()
    };
    let plan = plan_for(
        &[remove("jq", "apt")],
        &[Prior::Present(Some("1.6".into()))],
        &facts,
    );

    assert!(
        plan.steps.is_empty(),
        "re-installing it would hand the next sync the same work to do again (U41)"
    );
    assert!(plan.failures.is_empty());
}

#[test]
fn a_removal_in_a_run_with_no_manifest_is_put_back() {
    // A `rebuild`'s down phase and a hand-typed `uninstall` remove without reconciling, so
    // "nothing declares it" has no answer there — and the absence of an answer is not a reason
    // to leave the package gone. This is the other side of the test above.
    let facts = Facts {
        declared: None,
        ..Facts::default()
    };
    let plan = plan_for(
        &[remove("jq", "apt")],
        &[Prior::Present(Some("1.6".into()))],
        &facts,
    );

    assert_eq!(plan.steps.len(), 1, "the removal is compensated");
    assert!(matches!(plan.steps[0], Undo::ReinstateRemoved { .. }));
    assert!(plan.failures.is_empty());
}

#[test]
fn a_removal_of_something_that_was_never_there_undoes_nothing() {
    let plan = plan_for(&[remove("jq", "apt")], &[Prior::Absent], &Facts::default());

    assert!(plan.steps.is_empty());
    assert!(plan.failures.is_empty());
}

#[test]
fn the_plan_is_walked_in_reverse_so_the_machine_comes_back_the_way_it_went() {
    let facts = Facts {
        declared: Some(declared(&[])),
        ..Facts::default()
    };
    let plan = plan_for(
        &[install("first", "apt"), install("second", "apt")],
        &[Prior::Absent, Prior::Absent],
        &facts,
    );

    let names: Vec<&str> = plan.steps.iter().map(Undo::name).collect();
    assert_eq!(
        names,
        vec!["second", "first"],
        "the last thing done is the first thing undone"
    );
    // The backend travels with the name, and nothing asserted it: `Undo::backend` returning
    // `""` — or `"xyzzy"` — survived the whole suite, and a step with no backend is a rollback
    // that cannot find the manager to undo through (`registry.get(backend)` answers nothing and
    // the package is quietly left as it is).
    let backends: Vec<&str> = plan.steps.iter().map(Undo::backend).collect();
    assert_eq!(
        backends,
        vec!["apt", "apt"],
        "a step does not carry the manager it has to be undone through"
    );
}
