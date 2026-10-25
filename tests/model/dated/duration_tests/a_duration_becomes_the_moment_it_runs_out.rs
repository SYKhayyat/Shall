use super::*;

#[test]
fn a_duration_becomes_the_moment_it_runs_out() {
    // V.38: the file cannot hold "2 hours" — it would mean something different every
    // time it was read.
    assert_eq!(absolute_after(now(), "2h").unwrap(), "2026-07-17T14:00");
    assert_eq!(absolute_after(now(), "30m").unwrap(), "2026-07-17T12:30");
    assert_eq!(absolute_after(now(), "7d").unwrap(), "2026-07-24T12:00");
    assert_eq!(absolute_after(now(), "1w").unwrap(), "2026-07-24T12:00");
}
