use super::*;

#[test]
fn the_right_os_does_not_excuse_the_wrong_arch() {
    assert!(!linux64().accepts("tool-linux-arm64.tar.gz"));
}
