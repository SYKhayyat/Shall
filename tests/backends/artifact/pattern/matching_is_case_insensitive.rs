use super::*;

#[test]
fn matching_is_case_insensitive() {
    let p = AssetPattern::parse("*MUSL*").unwrap();
    assert!(p.matches("fd-musl-amd64.deb"));
}
