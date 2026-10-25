use super::*;

#[test]
fn nothing_locked_is_an_empty_slice_not_a_missing_answer() {
    assert!(ArtifactLedger::new().locked("foo/bar").is_empty());
}
