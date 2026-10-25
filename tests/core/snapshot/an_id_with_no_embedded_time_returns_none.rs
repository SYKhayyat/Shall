use super::*;

#[test]
fn an_id_with_no_embedded_time_returns_none() {
    // A snapshot Shall did not create, or a malformed id: no guess, so the caller falls
    // back rather than trusting a wrong time.
    assert!(Snapshot::time_from_id("some_manual_snapshot").is_none());
    assert!(Snapshot::time_from_id("tank/root@weekly-2026").is_none());
    // Right shape, non-numeric tail.
    assert!(Snapshot::time_from_id("shall_pre_sync_notadate12").is_none());
}
