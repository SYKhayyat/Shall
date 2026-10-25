use super::*;

#[test]
fn a_question_mark_matches_exactly_one_character() {
    let p = AssetPattern::parse("fd-?.deb").unwrap();
    assert!(p.matches("fd-1.deb"));
    assert!(!p.matches("fd-10.deb"));
}
