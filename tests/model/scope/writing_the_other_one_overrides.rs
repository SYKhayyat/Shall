use super::*;

#[test]
fn writing_the_other_one_overrides() {
    assert_eq!(Scope::resolve(Some("system"), Scope::User), Scope::System);
    assert_eq!(Scope::resolve(Some("user"), Scope::System), Scope::User);
}
