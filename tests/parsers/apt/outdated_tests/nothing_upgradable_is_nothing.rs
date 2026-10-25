use super::*;

#[test]
fn nothing_upgradable_is_nothing() {
    assert!(parse_apt_outdated("").is_empty());
    assert!(parse_apt_outdated("Listing...\n").is_empty());
}
