use super::*;

#[test]
fn a_matching_triple_is_accepted() {
    assert!(linux64().accepts("fd-v10.2.0-x86_64-unknown-linux-gnu.tar.gz"));
    assert!(linux64().accepts("fd_10.2.0_amd64.deb"));
}
