use super::*;

/// Editing the script changes its hash, so the new content's count is zero and it runs —
/// this is the whole idempotency model in one assertion.
#[test]
fn an_edited_script_is_a_different_content_and_runs_again() {
    let mut l = ExecLedger::new();
    l.record_run("hash-of-v1", "t".into(), "./s.sh", None);
    assert!(!Ceiling::read(None).permits(l.count("hash-of-v1")));
    assert!(Ceiling::read(None).permits(l.count("hash-of-v2")));
}
