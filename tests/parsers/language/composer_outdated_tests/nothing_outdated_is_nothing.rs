use super::*;

#[test]
fn nothing_outdated_is_nothing() {
    assert!(parse_composer_outdated("").is_empty());
    assert!(parse_composer_outdated("Changed current directory to /x\n").is_empty());
    assert!(parse_composer_outdated(r#"{"installed":[]}"#).is_empty());
}
