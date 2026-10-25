use super::*;

#[test]
fn a_pinned_version_matches_the_tag_with_or_without_a_v() {
    assert_eq!(tag_spellings("10.2.0"), tag_spellings("v10.2.0"));
    assert!(same_tag("10.2.0", "v10.2.0"));
    assert!(same_tag("v10.2.0", "10.2.0"));
    assert!(!same_tag("10.2.0", "10.2.1"));
}
