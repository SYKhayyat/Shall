use super::*;

#[test]
fn a_bare_name_matches_only_itself() {
    let p = AssetPattern::parse("fd_10.2.0_amd64.deb").unwrap();
    assert!(p.matches("fd_10.2.0_amd64.deb"));
    assert!(!p.matches("fd-musl_10.2.0_amd64.deb"));
}
