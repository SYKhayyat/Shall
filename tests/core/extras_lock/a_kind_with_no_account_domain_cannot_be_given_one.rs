use super::*;

#[test]
fn a_kind_with_no_account_domain_cannot_be_given_one() {
    for kind in [
        ResourceKind::Repo,
        ResourceKind::Firewall,
        ResourceKind::Link,
        ResourceKind::Dir,
        ResourceKind::Exec,
    ] {
        assert!(
            ExtraKey::with_user(kind, "x", Some("alice")).is_err(),
            "{kind} accepted an account"
        );
    }
}
