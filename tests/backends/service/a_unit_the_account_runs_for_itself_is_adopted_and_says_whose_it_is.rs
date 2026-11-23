use super::*;

/// A unit the invoking account runs for itself is adopted, and it says whose it is.
///
/// `#211`: `adopt` listed only the system domain, so an account's own units were silently
/// dropped from every manifest. They are not less real than a system unit — the same evidence,
/// read from the same init — and the mark matters as much as the line: `service:x@status=running`
/// without `@user=<account>` enables that unit for the whole machine, which is a different
/// service from the one that was adopted.
#[tokio::test]
async fn a_unit_the_account_runs_for_itself_is_adopted_and_says_whose_it_is() {
    let toml = r#"
[[init]]
name = "probe"
detect = "probe-detect"
list = ["probe-list-system", "--type=service"]
list_pattern = '^(\S+)'
list_strip_suffix = ".service"
start = [["probe-start", "{name}"]]
stop = [["probe-stop", "{name}"]]
user_list = ["probe-list-user", "--type=service"]
user_list_pattern = '^(\S+)'
user_list_strip_suffix = ".service"
user_start = [["probe-start-user", "{name}"]]
user_stop = [["probe-stop-user", "{name}"]]
"#;
    let file: InitProviderFile = toml::from_str(toml).unwrap();
    let mock = std::sync::Arc::new(MockExecutor::new(Arc::new(DashMap::new())));
    let mut machine = exit_of(0).unwrap();
    machine.stdout = b"sshd.service\n".to_vec();
    mock.set_response("probe-list-system --type=service", Ok(machine));
    let mut mine = exit_of(0).unwrap();
    mine.stdout = b"gpg-agent.service\n".to_vec();
    mock.set_response("probe-list-user --type=service", Ok(mine));
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
    let account = shall::core::account::Account::current().unwrap();

    let listed = q.list_manual().await.unwrap();
    let names: Vec<&str> = listed.iter().map(|p| p.name.as_str()).collect();
    assert!(
        names.contains(&"sshd") && names.contains(&"gpg-agent"),
        "both domains are listed, and each unit under its own name: {names:?}"
    );
    let mine = listed
        .iter()
        .find(|p| p.name == "gpg-agent")
        .expect("the account's own unit");
    assert_eq!(
        mine.properties.get("user").map(String::as_str),
        Some(account.name()),
        "the adopted line must say which account's domain the unit belongs to, or it is placed \
         for the whole machine instead"
    );
    let system = listed
        .iter()
        .find(|p| p.name == "sshd")
        .expect("the system unit");
    assert!(
        !system.properties.contains_key("user"),
        "a system unit carries no account mark: {system:?}"
    );
}

/// The two ways a second listing would offer the same unit twice, refused before they happen.
///
/// An init whose `user_list` is the same command as its `list` has no separate user domain
/// (launchd answers `launchctl list` for whoever runs it), and a name the system listing already
/// carries is the system's unit. Either one, offered twice, writes two lines for one name that
/// nothing downstream can tell apart.
#[tokio::test]
async fn one_unit_is_offered_once() {
    let toml = r#"
[[init]]
name = "probe"
detect = "probe-detect"
list = ["probe-list", "--type=service"]
list_pattern = '^(\S+)'
list_strip_suffix = ".service"
start = [["probe-start", "{name}"]]
stop = [["probe-stop", "{name}"]]
user_list = ["probe-list", "--type=service"]
user_list_pattern = '^(\S+)'
user_list_strip_suffix = ".service"
user_start = [["probe-start-user", "{name}"]]
user_stop = [["probe-stop-user", "{name}"]]
"#;
    let file: InitProviderFile = toml::from_str(toml).unwrap();
    let mock = std::sync::Arc::new(MockExecutor::new(Arc::new(DashMap::new())));
    let mut out = exit_of(0).unwrap();
    out.stdout = b"agent.service\n".to_vec();
    mock.set_response("probe-list --type=service", Ok(out));
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

    let listed = q.list_manual().await.unwrap();
    assert_eq!(listed.len(), 1, "one listing, one line: {listed:?}");
    assert!(
        !listed[0].properties.contains_key("user"),
        "the shared command is the system listing, so nothing is marked as an account's: {:?}",
        listed[0]
    );
}
