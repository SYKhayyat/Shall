use super::*;

/// Owner ruling: writing the default is accepted, not refused as redundant. A config may
/// state a thing it would also get for free.
#[test]
fn writing_the_default_is_accepted_and_means_the_same() {
    assert_eq!(Scope::resolve(Some("user"), Scope::User), Scope::User);
    assert_eq!(Scope::resolve(Some("system"), Scope::System), Scope::System);
}
