use super::*;

#[test]
fn a_dot_is_a_dot_and_not_any_character() {
    let p = AssetPattern::parse("fd.deb").unwrap();
    assert!(p.matches("fd.deb"));
    assert!(!p.matches("fdxdeb"));
}
