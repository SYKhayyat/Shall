use super::*;

#[test]
fn changing_the_asset_pattern_under_a_pin_asks_github_again() {
    let locked = [lock("v10.2.0", "fd-gnu.tar.gz", "tarball")];
    let state = installed("v10.2.0", &["fd-gnu.tar.gz"]);
    let known = Known {
        pin: Some("10.2.0"),
        locked: &locked,
        installed: Some(&state),
    };
    let musl = AssetPattern::parse("*musl*").unwrap();
    assert!(!answered_locally(&known, &tarballs(), Some(&musl)));
    let gnu = AssetPattern::parse("*gnu*").unwrap();
    assert!(answered_locally(&known, &tarballs(), Some(&gnu)));
}
