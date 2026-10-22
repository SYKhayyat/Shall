use super::*;

#[test]
fn a_bare_name_has_no_backend() {
    let Statement::Package(d) = p("ripgrep").unwrap() else {
        panic!()
    };
    assert_eq!(d.backend, None);
    assert_eq!(d.selector, Selector::Name("ripgrep".into()));
}
