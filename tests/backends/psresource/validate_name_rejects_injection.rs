use super::*;

#[test]
fn validate_name_rejects_injection() {
    assert!(validate_name("Pester'; rm -rf /; '").is_err());
    assert!(validate_name("foo bar").is_err());
    assert!(validate_name("$(evil)").is_err());
    assert!(validate_name("").is_err());
}
