use super::*;

#[test]
fn a_foreign_os_is_rejected() {
    assert!(!linux64().accepts("fd-v10.2.0-x86_64-pc-windows-msvc.zip"));
    assert!(!linux64().accepts("fd-v10.2.0-x86_64-apple-darwin.tar.gz"));
}
