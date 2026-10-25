use super::*;

#[test]
fn a_block_line_without_an_equals_is_an_error() {
    assert!(parse_block_line(&o(), "after_install ./setup.sh").is_err());
}
