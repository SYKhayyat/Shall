use super::*;

#[test]
fn a_path_holding_an_equals_sign_is_not_an_assignment() {
    assert!(!is_assignment("/opt/py=3/bin/python"));
    assert!(!is_assignment("=novalue"));
    assert!(!is_assignment("1BAD=x"));
    assert!(is_assignment("FOO=1"));
    assert!(is_assignment("PYTHONUNBUFFERED="));
}
