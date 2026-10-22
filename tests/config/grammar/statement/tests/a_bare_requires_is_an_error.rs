use super::*;

#[test]
fn a_bare_requires_is_an_error() {
    let err = p("apt:nginx@requires=libfoo").unwrap_err();
    assert!(err.what.contains("bare name"), "{}", err.what);
    assert!(err.hint.unwrap().contains("apt:libfoo"));
}
