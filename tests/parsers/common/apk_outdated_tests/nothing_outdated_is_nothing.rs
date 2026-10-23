use super::*;

#[test]
fn nothing_outdated_is_nothing() {
    assert!(parse_apk_outdated("", "apk").is_empty());
    assert!(parse_apk_outdated("Installed:  Available:\n", "apk").is_empty());
}
