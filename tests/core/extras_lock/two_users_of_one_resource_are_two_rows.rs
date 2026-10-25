use super::*;

#[test]
fn two_users_of_one_resource_are_two_rows() {
    for (kind, name) in [
        (ResourceKind::Shim, "rg"),
        (ResourceKind::Service, "nginx"),
        (ResourceKind::Schedule, "nightly"),
        (ResourceKind::Setting, "org.gnome.desktop/theme"),
    ] {
        let alice = account_scoped(kind, name, "alice");
        let bob = account_scoped(kind, name, "bob");
        assert_ne!(alice, bob, "{kind}: two accounts wrote one row");
        assert!(alice.ends_with("@user=alice"), "{alice}");
        assert!(bob.ends_with("@user=bob"), "{bob}");

        let mut ledger = ExtrasLedger::new();
        ledger.record(records(&[&alice]));
        assert_eq!(
            drifted(&ledger, &[&bob]),
            vec![alice.clone()],
            "{kind}: one account's row drifted on the other's declaration"
        );

        let parsed: ExtraKey = alice.parse().expect("the row reads back");
        assert_eq!(parsed.account_suffix(), Some("alice"), "{kind}");
        assert_eq!(parsed.subject, name, "{kind}: the subject is not the name");
    }
}
