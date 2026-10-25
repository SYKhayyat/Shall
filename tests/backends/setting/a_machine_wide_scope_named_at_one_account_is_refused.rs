use super::*;

#[test]
fn a_machine_wide_scope_named_at_one_account_is_refused() {
    let core = SettingBackendCore::new(CommandExecutor::new(true, false), adapters(vec![]));
    let registry = registry_like();
    let name = "org.gnome.x/theme";

    for (scope, user) in [
        (Scope::System, Some("alice")),
        (Scope::System, Some("someone-else")),
    ] {
        let err = core
            .refuse_system_scope_for_a_named_account(name, scope, user)
            .expect_err("a machine-wide value cannot also be one account's")
            .to_string();
        assert!(err.contains(name), "{err}");
        assert!(err.contains(user.unwrap()), "{err}");
        assert!(err.contains("@scope=system"), "{err}");
        assert!(err.contains("@user="), "{err}");
    }

    for (scope, user) in [
        (Scope::System, None),
        (Scope::User, Some("alice")),
        (Scope::User, None),
    ] {
        assert!(
            core.refuse_system_scope_for_a_named_account(name, scope, user)
                .is_ok(),
            "({scope:?}, {user:?}) is not a contradiction"
        );
    }

    let err = core
        .scope_of(&registry, Some("system"), name, Some("alice"))
        .expect_err("the apply must not reach the store with this line")
        .to_string();
    assert!(err.contains("nothing to write that is both"), "{err}");
}
