use super::*;

#[test]
fn a_missing_file_is_no_groups() {
    assert!(Groups::load(std::path::Path::new("no/such/groups"))
        .unwrap()
        .is_empty());
}
