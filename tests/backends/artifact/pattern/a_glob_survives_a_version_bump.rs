use super::*;

#[test]
fn a_glob_survives_a_version_bump() {
    let p = AssetPattern::parse("*musl*").unwrap();
    assert!(p.matches("fd-musl_10.2.0_amd64.deb"));
    assert!(p.matches("fd-musl_11.0.0_amd64.deb"));
    assert!(!p.matches("fd_10.2.0_amd64.deb"));
}
