use super::*;

#[test]
fn an_empty_option_list_is_an_error() {
    assert!(parse_short(&o(), "").is_err());
    assert!(parse_short(&o(), "   ").is_err());
}
