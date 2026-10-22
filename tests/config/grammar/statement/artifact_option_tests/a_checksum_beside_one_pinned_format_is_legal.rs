use super::*;

#[test]
fn a_checksum_beside_one_pinned_format_is_legal() {
    assert!(p("github:sharkdp/fd@sha256=abc,formats=deb").is_ok());
}
