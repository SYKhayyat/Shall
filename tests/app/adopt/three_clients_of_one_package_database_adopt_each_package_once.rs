use super::*;

/// Three clients of one database must adopt each package once, not once each.
///
/// Measured on the arch image before this held: 20 installed packages became 60
/// declarations, and the `uninstall jq` that followed planned three removals — pacman
/// removed jq and the two clients were then asked to remove a package that was gone,
/// which is `error: target not found: jq` and a sync that cannot converge again.
#[tokio::test]
async fn three_clients_of_one_package_database_adopt_each_package_once() {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs));
    let found = adopter(alpm_registry(mock, ["pacman", "yay", "paru"]))
        .discover()
        .await
        .unwrap();
    let keys: Vec<String> = found
        .adopt
        .iter()
        .map(|p| format!("{}:{}", p.backend, p.name))
        .collect();
    assert_eq!(keys, vec!["pacman:bash", "pacman:jq", "pacman:yay"]);
}
