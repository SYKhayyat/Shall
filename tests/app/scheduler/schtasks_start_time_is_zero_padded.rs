use super::*;

/// The reported defect: `0 3 * * *` produced `/ST 3:0`, and Task Scheduler answers
/// `ERROR: Invalid starttime value.` It wants `HH:mm`, zero-padded. Measured against real
/// `schtasks` on 2026-07-28: `/ST 3:0` is rejected at parse time, `/ST 03:00` is accepted
/// and reaches the privilege check.
#[test]
fn schtasks_start_time_is_zero_padded() {
    assert_eq!(
        map_cron_to_schtasks("0 3 * * *").unwrap(),
        vec!["/SC", "DAILY", "/ST", "03:00"]
    );
}
