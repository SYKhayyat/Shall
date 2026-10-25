use super::*;

#[test]
fn vars_shall_is_the_embedded_kind() {
    assert_eq!(kind_of("vars.shall"), Some(Kind::Embedded));
    assert_eq!(kind_of("vars"), Some(Kind::LineFile));
    assert_eq!(kind_of("vars.py"), Some(Kind::External));
    assert_eq!(kind_of("modules"), None);
}
