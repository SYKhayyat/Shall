use super::*;

#[test]
fn a_stray_closing_brace_is_an_error() {
    assert!(doc("apt:curl\n}\n").is_err());
}
