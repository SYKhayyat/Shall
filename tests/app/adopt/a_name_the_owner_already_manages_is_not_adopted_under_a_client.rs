use super::*;

/// A package already declared under the database's owner is not re-adopted under a client.
///
/// This is the exact shape the container hit: `install jq` writes `pacman:jq`, so pacman
/// skipped it as already managed — and the two clients, filtered separately, adopted the
/// same jq under their own names. The name has to be claimed by the skip as well as by
/// the take.
#[tokio::test]
async fn a_name_the_owner_already_manages_is_not_adopted_under_a_client() {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs));
    let reg = alpm_registry(mock, ["pacman", "yay", "paru"]);
    let config = Config {
        guard: shall::config::GuardSettings {
            protected_packages: vec![],
            ..Default::default()
        },
        ..Config::default()
    };
    let mut state = StateRegistry::default();
    state.add("pacman", "jq", None, Default::default(), "test", false);
    let m = Adopter::new(over(reg), Arc::new(Mutex::new(state)), &config);
    let keys: Vec<String> = m
        .discover()
        .await
        .unwrap()
        .adopt
        .iter()
        .map(|p| format!("{}:{}", p.backend, p.name))
        .collect();
    assert_eq!(keys, vec!["pacman:bash", "pacman:yay"]);
}
