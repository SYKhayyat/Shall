use super::*;

#[test]
fn a_qualified_requires_is_accepted() {
    assert!(p("apt:nginx@requires=apt:libfoo").is_ok());
}
