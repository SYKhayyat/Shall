use super::*;

#[test]
fn a_stray_brace_is_an_error() {
    assert!(parse("apt\n}\n").is_err());
}
