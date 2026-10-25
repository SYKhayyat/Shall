use super::*;

/// The gate behind the ruling. Every path that declines to measure says so, and this fails if
/// one of them goes back to silence.
///
/// **Asserted by what each marker says, not by how many there are.** The first version
/// counted, and it failed on the first case for the most useful reason available: `sync` is
/// `Mutating`, which carries neither a budget nor a shape, so `report_if_over` emitted **two**
/// markers and the count said one. A count here would have been fixed by weakening the
/// second emission, which is the defect the mechanism exists to prevent. Each case below
/// drives one path and asserts the sentence that path is responsible for.
#[test]
fn unmeasured_is_reported_on_every_skip_path() {
    // (1) `report_if_over`: this class carries no wall-clock budget. `sync` is `Mutating`, so
    // it declines on both counts at once — which is why this asserts the sentence and not
    // the number.
    let lines = captured(|| {
        report_if_over("sync", Duration::from_secs(1));
    });
    let got = markers(&lines);
    assert!(
        got.iter().any(|m| m.contains("wall-clock budget")),
        "the budget skip must say so: {got:?}"
    );
    assert!(
        got.iter().any(|m| m.contains("no fan-out shape budget")),
        "and the shape skip on the same call must say so too: {got:?}"
    );
    assert!(
        got.iter().all(|m| m.contains("sync")),
        "every marker names the command it is about: {got:?}"
    );

    // (2) `report_shape` on a class with no fan-out shape at all, and only one reason to
    // decline — so this one *is* exactly one marker, and is the negative for the case above.
    let lines = captured(|| {
        report_shape("info", Class::OneBackend);
    });
    let got = markers(&lines);
    assert_eq!(got.len(), 1, "one reason to decline, one marker: {got:?}");
    assert!(got[0].contains("no fan-out shape budget"), "{got:?}");

    // (3) The rule ran and the run was within it — a PASS, and the one `None` in these
    // functions that must stay silent. If this ever starts emitting, the marker has stopped
    // meaning "not measured" and a green run is being described as unmeasured.
    let shape = Class::EveryBackend
        .shape()
        .expect("the fan-out class has a shape");
    assert!(
        shape_violation(
            shape,
            16,
            Duration::from_secs(16),
            Duration::from_secs(8),
            Duration::from_secs(8),
            2
        )
        .is_none(),
        "the healthy reading is within budget, so there is nothing to report"
    );
    let lines = captured(|| {
        report_shape("list", Class::OneBackend);
    });
    assert!(
        markers(&lines).is_empty() || !markers(&lines).iter().any(|m| m.contains("within")),
        "a pass is not a skip"
    );

    // (4) The two conditions the third marker is guarded by. They are asserted through the
    // shared predicates rather than by driving `report_shape`, because reaching that arm
    // needs `core::timing::enable()` — a one-way process global with no reset, so a unit
    // test that called it would make every later test in the binary order-dependent. The
    // condition and its marker are guarded by the SAME `is_measurable`, which is the whole
    // reason it is a named function: the rule and the reporter cannot come to disagree.
    assert!(
        !shape.is_measurable(shape.min_children - 1),
        "one fewer is not a measurement"
    );
    assert!(
        shape.is_measurable(shape.min_children),
        "and the floor itself is"
    );
    // And the same predicate answers the *other* way there is no measurement, so the two
    // callers of `shape_measurement_gap` are guarded by one question rather than two that
    // can drift apart — which is the failure this file's own comment above calls the worst
    // kind it keeps finding.
    assert!(
        shape_measurement_gap(
            shape,
            shape.min_children,
            Duration::from_secs(10),
            Duration::from_secs(9),
            1,
        )
        .is_some(),
        "enough children, one holding 90% of the time, and one wave — leaves no ratio to read"
    );

    // (5) The scheduling rule, at its own site: a plan with one package has no reachable
    // violation, and a plan inside the rule is silent while one outside it is not.
    assert!(!scheduling_is_measurable(1, 1));
    assert!(!scheduling_is_measurable(2, 0));
    assert!(scheduling_is_measurable(2, 1));
    assert!(
        scheduling_violation(1, 1, 9).is_none(),
        "unmeasurable, so not a violation"
    );
    assert!(
        scheduling_violation(20, 1, 20).is_some(),
        "a serial loop is caught"
    );
    assert!(
        scheduling_violation(20, 4, 4).is_none(),
        "one wave per level is the plan's shape, not a regression"
    );
}
