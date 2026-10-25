use super::*;

#[test]
fn a_line_that_is_not_a_backend_name_is_an_error() {
    assert!(parse("apt install curl\n").is_err());
}
