use super::*;

/// Absent means the store's own default — that is what makes `@scope=` an override rather
/// than a thing every line must carry.
#[test]
fn an_unwritten_scope_is_the_default() {
    assert_eq!(Scope::resolve(None, Scope::User), Scope::User);
    assert_eq!(Scope::resolve(None, Scope::System), Scope::System);
}
