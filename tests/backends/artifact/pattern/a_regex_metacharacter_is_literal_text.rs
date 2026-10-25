use super::*;

#[test]
fn a_regex_metacharacter_is_literal_text() {
    let p = AssetPattern::parse("tool+1.zip").unwrap();
    assert!(p.matches("tool+1.zip"));
    assert!(!p.matches("toolll1.zip"));
}
