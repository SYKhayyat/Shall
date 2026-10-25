use super::*;

#[tokio::test]
async fn a_per_user_listing_comes_from_the_per_user_row() {
    let toml = r#"
[[init]]
name = "probe"
detect = "probe-detect"
start = [["probe-start", "{name}"]]
stop = [["probe-stop", "{name}"]]
user_start = [["probe-start-user", "{name}"]]
user_stop = [["probe-stop-user", "{name}"]]
list = ["probe-list-system", "--type=service"]
list_pattern = '^(\S+)'
list_strip_suffix = ".service"
user_list = ["probe-list-user", "--user", "--type=service"]
user_list_pattern = '^(\S+)'
user_list_strip_suffix = ".service"
"#;
    let file: InitProviderFile = toml::from_str(toml).unwrap();
    let mock = std::sync::Arc::new(MockExecutor::new(Arc::new(DashMap::new())));
    let account = shall::core::account::Account::current().unwrap();
    let mut alice = exit_of(0).unwrap();
    alice.stdout = b"alice.service loaded\n".to_vec();
    mock.set_response("probe-list-user --user --type=service", Ok(alice));
    let mut machine = exit_of(0).unwrap();
    machine.stdout = b"machine.service loaded\n".to_vec();
    mock.set_response("probe-list-system --type=service", Ok(machine));
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        Arc::new(DashMap::new()),
        Arc::new(DashMap::new()),
    );
    let core = ServiceBackendCore::with_providers(exec.clone(), file.init);
    let q = ServiceQueryable {
        core: Arc::new(core),
    };

    let per_user = q.list_installed_for(Some(account.name())).await.unwrap();
    assert!(
        per_user.iter().any(|p| p.name == "alice"),
        "{per_user:?}: the per-user listing did not come from the per-user row"
    );
    assert!(
        !per_user.iter().any(|p| p.name == "machine"),
        "{per_user:?}: the per-user listing answered with the machine's services"
    );

    let system = q.list_installed_for(None).await.unwrap();
    assert!(
        system.iter().any(|p| p.name == "machine"),
        "{system:?}: the system listing did not come from the system row"
    );
}
