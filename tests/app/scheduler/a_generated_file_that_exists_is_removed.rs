use super::*;

#[test]
fn a_generated_file_that_exists_is_removed() {
    let dir = tempfile::tempdir().unwrap();
    let unit = dir.path().join("shall-nightly.timer");
    std::fs::write(&unit, "[Timer]\n").unwrap();
    remove_generated(&unit).unwrap();
    assert!(!unit.exists());
}
