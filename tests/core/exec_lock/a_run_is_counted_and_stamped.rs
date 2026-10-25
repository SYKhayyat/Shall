use super::*;

#[test]
fn a_run_is_counted_and_stamped() {
    let mut l = ExecLedger::new();
    l.record_run("abc", "2026-07-24T00:00:00Z".into(), "./s.sh", None);
    assert_eq!(l.count("abc"), 1);
    l.record_run("abc", "2026-07-24T01:00:00Z".into(), "./s.sh", None);
    assert_eq!(l.count("abc"), 2);
    // A second content is its own count — the hash is the identity.
    assert_eq!(l.count("xyz"), 0);
}
