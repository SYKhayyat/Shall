use super::*;

#[test]
fn win_does_not_match_inside_a_longer_word() {
    assert!(linux64().accepts("winnowing-tool-linux.tar.gz"));
}
