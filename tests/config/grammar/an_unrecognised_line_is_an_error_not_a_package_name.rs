use super::*;

#[test]
fn an_unrecognised_line_is_an_error_not_a_package_name() {
    // II.2, and VI.1's "any typo becomes a package name". The error must name the
    // file, the line, and what was expected.
    let err = doc("apt:curl\nthis is not a thing\n").unwrap_err();
    assert_eq!(err.origin.line, 2);
    assert!(err.to_string().contains("modules/dev.txt:2"), "{}", err);
}
