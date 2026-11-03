use super::*;

/// **"This run had N slots and used M"** — the sentence `#111` says no instrument in the tree
/// could write.
///
/// `#107` made the shape verdict judge on width, because the ratio collapses when starting a
/// child costs what the child costs. The cost of that change, in `latency.rs`'s own words, was
/// that *"a fan-out that uses two slots where it had sixteen is no longer reported"*: both runs
/// are two wide, and the only number that could tell them apart — the budget — was an ambient
/// fact about the machine rather than a value the reporter could read.
///
/// So the reading is now pure and takes the budget. It answers *the degree it lost*, which is
/// what the issue's second half is about, and it does not judge: a fan-out that overlapped is
/// still not a collapse, and the collapse verdict stays with `shape_violation`.
#[test]
fn a_wide_run_says_how_many_slots_it_had_and_how_many_it_used() {
    let reading = slot_reading(3, 16, 17).expect("a three-wide run overlapped");
    assert!(
        reading.contains("16 slot(s)"),
        "the reading has to name the budget, or it cannot be compared with anything: {reading}"
    );
    assert!(
        reading.contains("used 3 of them"),
        "and the degree it reached: {reading}"
    );
    assert!(
        reading.contains("17 child command"),
        "over how many children, which is what says whether the budget was even the constraint: \
         {reading}"
    );
    assert!(
        reading.contains("never reached the width it was allowed"),
        "three of sixteen slots over seventeen children is the fan-out losing width, and the \
         reading says so: {reading}"
    );
}

/// **Saturation is not a loss**, and this is the control that keeps the sentence useful: a run
/// that used every slot it had was bounded by its own budget, not by a scheduler.
#[test]
fn a_run_that_used_every_slot_is_not_described_as_losing_one() {
    let reading = slot_reading(16, 16, 17).expect("a sixteen-wide run overlapped");
    assert!(reading.contains("used 16 of them"), "{reading}");
    assert!(
        !reading.contains("never reached the width"),
        "a run at its ceiling used every slot it had: {reading}"
    );
}

/// **Fewer children than slots is not a loss either.** Seven children over sixteen slots reach
/// seven, which is all there was to ask — the fan-out was one wave and the extra slots were
/// never needed.
#[test]
fn fewer_children_than_slots_is_not_a_lost_slot() {
    let reading = slot_reading(7, 16, 7).expect("seven wide over seven children overlapped");
    assert!(reading.contains("used 7 of them"), "{reading}");
    assert!(
        !reading.contains("never reached the width"),
        "there were seven questions and seven answers: {reading}"
    );
}

/// **A one-wide run is not this reading's business.** It is a serial fan-out, and that verdict
/// belongs to `shape_violation` — which says it once, in the words the report already uses. Two
/// sentences for one defect is how a reader learns to skim.
#[test]
fn a_serial_run_is_left_to_the_collapse_verdict() {
    assert!(
        slot_reading(1, 16, 17).is_none(),
        "a serial fan-out must not also be described as a slot reading"
    );
    assert!(slot_reading(0, 16, 0).is_none(), "no children, no reading");
}

/// The budget the reporter is handed is the run's child-process budget, and `0` cannot reach it
/// (`Config::from_file` refuses it), but a caller that forgot would otherwise divide a reading
/// by nothing: `max(1)` is the same clamp every fan-out applies.
#[test]
fn a_budget_of_zero_still_reads_as_a_budget_of_one() {
    let reading = slot_reading(2, 0, 5).expect("two wide");
    assert!(reading.contains("1 slot(s)"), "{reading}");
}
