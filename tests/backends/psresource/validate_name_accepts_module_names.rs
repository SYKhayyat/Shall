use super::*;

#[test]
fn validate_name_accepts_module_names() {
    assert!(validate_name("Pester").is_ok());
    assert!(validate_name("PSReadLine").is_ok());
    assert!(validate_name("Az.Accounts").is_ok());
}
