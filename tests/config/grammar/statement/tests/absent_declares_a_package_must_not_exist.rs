use super::*;

#[test]
fn absent_declares_a_package_must_not_exist() {
    let Statement::Absent(d) = p("absent:apt:libreoffice").unwrap() else {
        panic!()
    };
    assert_eq!(d.backend.as_deref(), Some("apt"));
    assert_eq!(d.selector.as_str(), "libreoffice");
}
