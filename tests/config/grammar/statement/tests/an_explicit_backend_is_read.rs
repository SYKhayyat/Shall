use super::*;

#[test]
fn an_explicit_backend_is_read() {
    let Statement::Package(d) = p("apt:curl").unwrap() else {
        panic!()
    };
    assert_eq!(d.backend.as_deref(), Some("apt"));
    assert_eq!(d.selector, Selector::Name("curl".into()));
}
