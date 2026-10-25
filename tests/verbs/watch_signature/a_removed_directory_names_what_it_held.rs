use super::*;

/// A whole directory going is one event with every file in it named.
#[tokio::test]
async fn a_removed_directory_names_what_it_held() {
    let dir = repo().await;
    for name in ["backends.toml", "firewall.toml"] {
        std::fs::write(dir.path().join("adapters").join(name), "").expect("a row");
    }
    let before = signature(dir.path()).await;
    std::fs::remove_dir_all(dir.path().join("adapters")).expect("the removal");
    assert_eq!(
        changed(&before, &signature(dir.path()).await),
        vec![
            "adapters/backends.toml".to_string(),
            "adapters/firewall.toml".to_string()
        ]
    );
}
