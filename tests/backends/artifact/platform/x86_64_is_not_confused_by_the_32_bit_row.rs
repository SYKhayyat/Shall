use super::*;

#[test]
fn x86_64_is_not_confused_by_the_32_bit_row() {
    assert!(linux64().accepts("tool-linux-x86_64.tar.gz"));
    assert!(!linux64().accepts("tool-linux-x86.tar.gz"));
}
