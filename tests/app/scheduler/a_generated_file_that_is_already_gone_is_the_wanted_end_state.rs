use super::*;

#[test]
fn a_generated_file_that_is_already_gone_is_the_wanted_end_state() {
    let dir = tempfile::tempdir().unwrap();
    assert!(remove_generated(&dir.path().join("shall-nightly.timer")).is_ok());
}
