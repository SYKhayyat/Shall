use super::*;

#[test]
fn a_regex_selects_by_pattern() {
    let Statement::Package(d) = p("apt:re:^fonts-").unwrap() else {
        panic!()
    };
    assert_eq!(d.backend.as_deref(), Some("apt"));
    assert_eq!(d.selector, Selector::Regex("^fonts-".into()));
}
