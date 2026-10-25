use super::*;

#[test]
fn arm_does_not_match_inside_armv7() {
    let arm64 = Platform::new("linux", "aarch64");
    assert!(!arm64.accepts("tool-linux-armv7.tar.gz"));
    assert!(arm64.accepts("tool-linux-arm64.tar.gz"));
}
