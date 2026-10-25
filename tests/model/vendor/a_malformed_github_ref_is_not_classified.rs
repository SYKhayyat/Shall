use super::*;

#[test]
fn a_malformed_github_ref_is_not_classified() {
    assert_eq!(Source::classify("github:noshslash"), None);
    assert_eq!(Source::classify("github:a/b/c"), None);
    assert_eq!(Source::classify("github:/b"), None);
}
