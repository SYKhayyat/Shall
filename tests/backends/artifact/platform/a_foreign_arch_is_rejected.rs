use super::*;

#[test]
fn a_foreign_arch_is_rejected() {
    assert!(!linux64().accepts("fd_10.2.0_arm64.deb"));
    assert!(!linux64().accepts("fd-v10.2.0-i686-unknown-linux-gnu.tar.gz"));
}
