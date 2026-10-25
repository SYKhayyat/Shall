use super::*;

/// The bound `Mutating` is measured by, at and either side of its edge.
///
/// Named cases rather than one: `waves == depth` is the perfect scheduler and must pass,
/// `waves == packages` is the serial loop and must fail, and the two meet on a chain —
/// where a serial loop *is* the correct schedule and must not be reported. That last one is
/// the case a tuned threshold would get wrong and this rule gets right for free, because it
/// compares against the plan rather than against a number read off a host. Waves *below*
/// depth are the healthy overlap case and belong here for the same reason.
#[test]
fn a_serialised_plan_is_out_of_budget_and_a_deep_one_is_not() {
    // (packages, depth, waves, should_report, what this is)
    for (packages, depth, waves, reported, what) in [
        (20, 1, 1, false, "twenty independent packages, one wave"),
        (20, 4, 4, false, "four levels, four waves — perfect"),
        (20, 20, 20, false, "a chain: serial is the correct schedule"),
        (
            20,
            4,
            2,
            false,
            "independent chains overlapping, under depth",
        ),
        (2, 1, 1, false, "the smallest plan that can be measured"),
        (20, 1, 20, true, "one level handed out one at a time"),
        (20, 4, 20, true, "four levels handed out one at a time"),
        (
            20,
            4,
            5,
            true,
            "one idle restart more than the plan requires",
        ),
        (1, 1, 1, false, "a single package has nothing to schedule"),
    ] {
        let got = scheduling_violation(packages, depth, waves);
        assert_eq!(
            got.is_some(),
            reported,
            "{what}: {packages} package(s), depth {depth}, {waves} wave(s) — got {got:?}"
        );
    }

    let serial = scheduling_violation(20, 1, 20).expect("just asserted");
    assert!(serial.contains("19 more time(s)"), "{serial}");
}
