use super::*;

/// The recorder is global, so these run as one test rather than racing each other over it.
#[test]
fn a_run_is_gathered_per_label_slowest_first_and_off_by_default() {
    // Off by default: `begin` hands back nothing, and `end` records nothing from it.
    assert!(!is_enabled());
    assert!(begin().is_none());
    end(None, "winget", &["list".to_string()]);
    assert!(
        summary().0.is_empty(),
        "a disabled recorder recorded a span"
    );

    enable();
    assert!(is_enabled());

    // Given, not slept for. The durations below are the test's subject matter; producing
    // them with a sleep made the assertions a measurement of the host's scheduler.
    let ms = Duration::from_millis;
    record("winget", &["list".to_string()], ms(0), ms(30));
    record(
        "cargo",
        &["install".to_string(), "--list".to_string()],
        ms(30),
        ms(5),
    );
    record(
        "cargo",
        &["install".to_string(), "--list".to_string()],
        ms(35),
        ms(5),
    );
    // No args at all still names the program, rather than producing an empty row.
    record("emacs", &[], ms(40), ms(1));

    let (rows, _, summed) = summary();
    assert_eq!(rows.len(), 3, "three distinct labels, not one row per call");

    // Slowest first.
    assert_eq!(rows[0].label, "winget list");
    assert_eq!(rows[0].calls, 1);

    // Keyed by program + FIRST arg, so `--list` does not open a second row, and the two
    // calls gather into one.
    let cargo = rows
        .iter()
        .find(|r| r.label == "cargo install")
        .expect("cargo install row");
    assert_eq!(cargo.calls, 2);
    assert!(
        cargo.total >= cargo.longest,
        "total must include the longest"
    );

    assert!(rows.iter().any(|r| r.label == "emacs"));

    assert!(summed >= rows[0].total, "the sum covers every row");

    // Every span above ran to completion before the next began, so each is its own wave.
    // This is the reading that matters: same rows, same totals, and a serial run is only
    // distinguishable from an overlapped one here.
    assert_eq!(waves(), 4, "four children, none overlapping, is four waves");
    // And the half that survives a slow host: a hand-off is not overlap, so a run that
    // never had two children in flight is 1 wide however the ratio reads.
    assert_eq!(
        width(),
        1,
        "a child that starts as the last one ends was not in flight alongside it"
    );

    let winget = rows.iter().find(|r| r.label == "winget list").unwrap();
    assert!(
        winget.last_end >= winget.first_at + winget.longest,
        "a row's span must cover its own longest call"
    );

    end_records_the_time_that_actually_passed();
}
