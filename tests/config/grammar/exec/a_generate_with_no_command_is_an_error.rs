use super::*;

#[test]
fn a_generate_with_no_command_is_an_error() {
    assert!(pv("generate:").is_err());
}
