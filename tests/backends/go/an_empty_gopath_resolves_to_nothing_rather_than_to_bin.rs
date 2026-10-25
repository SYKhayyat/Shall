use super::*;

#[test]
fn an_empty_gopath_resolves_to_nothing_rather_than_to_bin() {
    assert!(gopath_bin("").is_none());
    assert!(gopath_bin("   \n").is_none());
}
