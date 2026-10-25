use super::*;

#[test]
fn an_unknown_content_has_never_run() {
    assert_eq!(ExecLedger::new().count("deadbeef"), 0);
}
