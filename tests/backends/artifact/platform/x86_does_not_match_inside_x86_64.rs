use super::*;

#[test]
fn x86_does_not_match_inside_x86_64() {
    let x86 = Platform::new("linux", "x86");
    assert!(!x86.accepts("tool-linux-x86_64.tar.gz"));
    assert!(x86.accepts("tool-linux-i686.tar.gz"));
}
