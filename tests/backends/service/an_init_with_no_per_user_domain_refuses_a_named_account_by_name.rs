use super::*;

#[tokio::test]
async fn an_init_with_no_per_user_domain_refuses_a_named_account_by_name() {
    let account = shall::core::account::Account::current().unwrap();
    for name in ["openrc", "sysvinit", "windows-sc"] {
        let toml: InitProviderFile = toml::from_str(&format!(
            r#"
[[init]]
name = "{name}"
detect = "probe-detect"
enable = [["probe-enable", "{{name}}"]]
disable = [["probe-disable", "{{name}}"]]
start = [["probe-start", "{{name}}"]]
stop = [["probe-stop", "{{name}}"]]
"#
        ))
        .unwrap();
        let (mock, exec) = service_layer(&[]);
        let core = ServiceBackendCore::with_providers(exec, toml.init);
        let err = core
            .apply_in(ServiceAction::Start, "web", false, Some(account.name()))
            .await
            .expect_err("a per-user line against a system-only init must be refused");
        let err = err.to_string();
        assert!(
            err.contains(name),
            "the refusal does not name the init: {err}"
        );
        assert!(err.contains(account.name()), "no account named: {err}");
        assert!(err.contains("Nothing was changed"), "{err}");
        assert!(
            mock.get_calls().await.is_empty(),
            "a refused per-user line still ran {:?}",
            mock.get_calls().await
        );
    }
}
