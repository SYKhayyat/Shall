// tests/app/sync/planner/an_advice_a_script_can_read.rs
//
// **A `--json` consumer should not have to read English** (`#18`, finding 7).
//
// The advice for every kind of skip was already written — `SkipKind::advice` — but only ever
// reached a terminal. A script reading `sync --dry-run --json` saw `kind: "install_skipped"` and a
// sentence describing the *situation*, and had to map the enum to an action itself. That is the
// same defect `heal` was cured of (typed objections rather than parsing guard prose), one surface
// over: the machine-readable answer required reading prose.
//
// The field is derived from the kind rather than stored per row, so it cannot drift from the
// heading and the human sentence that already belong to that kind. These tests hold that line in
// both directions: every kind an advice for the terminal names has one in the document too, and
// the two spellings come from one source.

use shall::app::sync::planner::{SkipKind, Skipped};

/// The four kinds, named here so a fifth cannot be added without this file mentioning it.
const ALL_KINDS: &[SkipKind] = &[
    SkipKind::RemovalDeclined,
    SkipKind::InstallSkipped,
    SkipKind::RemovalWithdrawn,
    SkipKind::CouldNotAsk,
];

fn row(kind: SkipKind) -> Skipped {
    Skipped {
        key: "cargo:hello".into(),
        reason: "whatever the manager said".into(),
        advice: kind.advice().to_string(),
        kind,
    }
}

/// **Every kind carries advice in the document**, and the advice is the kind's own words.
#[test]
fn every_skip_kind_carries_its_advice_into_the_json() {
    for kind in ALL_KINDS {
        let serialized = serde_json::to_value(row(*kind)).expect("a skip serializes");

        let advice = serialized
            .get("advice")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                panic!(
                    "a `{:?}` skip carries no `advice` field: {serialized}",
                    kind
                )
            });

        assert_eq!(
            advice,
            kind.advice(),
            "the document's advice for `{kind:?}` is not the one the terminal prints, so a script \
             and a person were told different things"
        );
        assert!(
            !advice.is_empty(),
            "a `{kind:?}` skip carries an empty advice string, which is a field that exists and \
             answers nothing"
        );
    }
}

/// **The advice is the action, not a restatement of the situation.**
///
/// `reason` describes what happened; `advice` says what to do. They are separate fields because a
/// script that wants to log the situation and act on the fix needs both, and a document where the
/// two are the same sentence offers only one.
#[test]
fn the_advice_is_not_a_copy_of_the_reason() {
    for kind in ALL_KINDS {
        let row = row(*kind);
        assert_ne!(
            row.advice, row.reason,
            "a `{kind:?}` skip repeats its reason as its advice, so the field a script acts on \
             says nothing it did not already say"
        );
    }
}

/// **The kinds are told apart**, which is the property the whole field rests on: two kinds with one
/// advice would be a field that cannot decide anything.
#[test]
fn no_two_kinds_share_an_advice() {
    let mut seen: Vec<(&str, SkipKind)> = Vec::new();
    for kind in ALL_KINDS {
        let advice = kind.advice();
        if let Some((_, other)) = seen.iter().find(|(a, _)| *a == advice) {
            panic!("`{kind:?}` and `{other:?}` share the advice {advice:?}");
        }
        seen.push((advice, *kind));
    }
}

/// **The planner fills it in, not only the struct's own constructor.**
///
/// The three tests above build a `Skipped` by hand, so they would all pass on a planner that left
/// the field empty everywhere. This one runs a real plan whose install cannot happen — a backend
/// that is not on this machine — and reads the document the user's script would read.
#[tokio::test]
async fn a_real_plan_puts_advice_in_the_document() {
    use shall::app::sync::planner::{ChangePlanner, PlanScope};
    use shall::core::PackageSpec;
    use std::collections::HashMap;

    let kernel = crate::mock_providers::TestKernel::new().await;

    // A manager this machine does not have. `runs_here` is what decides, and asking it is the
    // same question the planner asks.
    // `runs_here` is `get(name).is_some_and(|b| b.is_available())`, so a name the registry does
    // not hold is a manager this machine cannot act through — which is exactly the row
    // `BackendNotOnThisMachine` is written for. Using the mock's own list would not do: it
    // declares every manager present, so nothing in it answers `false`.
    let absent = "a-manager-that-is-not-registered".to_string();
    assert!(!kernel.app.registry.runs_here(&absent));

    let mut desired: HashMap<String, Vec<PackageSpec>> = HashMap::new();
    desired.insert(
        absent.clone(),
        vec![PackageSpec {
            name: "anything-at-all".into(),
            backend: absent,
            options: Default::default(),
            requires: vec![],
            present: true,
        }],
    );

    let report = {
        let state = kernel.app.state.lock().await;
        ChangePlanner::new(kernel.app.registry.clone(), &state, &kernel.app.config)
            .plan(&desired, PlanScope::Whole(Default::default()))
            .await
            .expect("the plan")
            .generate_report()
    };

    // The plan may also have nothing to skip on a machine where every known manager happens to
    // be installed; then the fixture cannot make the claim and says so rather than passing.
    assert!(
        !report.skipped.is_empty(),
        "this fixture could not produce a skip, so it proves nothing about the document: {report:?}"
    );

    let document = serde_json::to_value(&report).expect("the report serializes");
    let rows = document["skipped"]
        .as_array()
        .expect("skipped rows in the document");
    for row in rows {
        let advice = row
            .get("advice")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("a real plan's skip carries no `advice`: {row}"));
        assert!(
            !advice.is_empty(),
            "a real plan put an empty advice string in the document: {row}"
        );
    }
}
