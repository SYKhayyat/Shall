use super::*;

#[test]
fn validate_version_accepts_semver_like() {
    assert!(validate_version("2.10.3").is_ok());
    assert!(validate_version("1.0.0-beta1").is_ok());
    assert!(validate_version("1.0'; evil").is_err());
}
