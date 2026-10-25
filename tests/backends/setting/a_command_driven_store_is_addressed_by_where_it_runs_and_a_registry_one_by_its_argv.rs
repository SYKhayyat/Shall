use super::*;

#[test]
fn a_command_driven_store_is_addressed_by_where_it_runs_and_a_registry_one_by_its_argv() {
    let gs = gsettings();
    let unscoped = StoreTarget { account: None };
    let here = StoreTarget {
        account: Some(shall::core::account::Account::current().unwrap()),
    };
    for (label, target) in [("unscoped", &unscoped), ("this account", &here)] {
        let (_, user) = gs.read_command_for(Scope::User, "org.gnome.x", "k", target);
        let (_, system) = gs.read_command_for(Scope::System, "org.gnome.x", "k", target);
        assert_eq!(user, vec!["get", "org.gnome.x", "k"], "{label}");
        assert_eq!(system, user, "{label}: gsettings has no machine-wide store");
    }

    let reg = registry();
    let (_, plain) = reg.read_command_for(Scope::User, "Software\\X", "k", &unscoped);
    let (_, named) = reg.read_command_for(Scope::User, "Software\\X", "k", &here);
    assert!(
        plain.iter().any(|a| a.contains("HKCU")),
        "an unscoped registry read is not HKCU: {plain:?}"
    );
    if here.is_addressable(&reg) {
        assert_ne!(
            plain, named,
            "a registry read for a named account is byte-identical to the unscoped one, so \
             it would address the invoking account's hive"
        );
        assert!(
            named.iter().any(|a| a.contains("HKEY_USERS")),
            "a named registry read does not address that account's own hive: {named:?}"
        );
    }
}
