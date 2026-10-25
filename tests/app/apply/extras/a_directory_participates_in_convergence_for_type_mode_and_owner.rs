use super::*;

#[tokio::test]
async fn a_directory_participates_in_convergence_for_type_mode_and_owner() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("state");
    let account = shall::core::account::Account::current().unwrap();
    let mut options = Options::default();
    options.set("mode", "0700");
    options.set("owner", account.name());
    let statement = Statement::Dir(path.to_string_lossy().to_string(), options);
    let key = ExtraKey::new(
        shall::config::grammar::ResourceKind::Dir,
        path.display().to_string(),
    );
    let config = Arc::new(shall::config::Config::default());
    let registry = BackendRegistry::new();
    let executor = CommandExecutor::new(false, false);
    std::fs::create_dir(&path).unwrap();

    // The mode half is a Unix fact, and this is the platform that can hold it.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(
            in_effect(&config, &registry, &executor, &statement, &key).await,
            Some(true),
            "a 0700 directory owned by the account the line names is converged"
        );
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(
            in_effect(&config, &registry, &executor, &statement, &key).await,
            Some(false),
            "0755 and 0700 are different modes, and the wider one is not what was declared"
        );
    }
    // **And the answer off Unix is `Some(false)`, asserted rather than skipped.** A `dir:`
    // carrying a mode cannot be confirmed where there are no mode bits, and `Some(false)` is
    // the product's own choice: `None` is *unverifiable*, which places, so `sync` would
    // re-apply the line on every run while `check` filed it under `ok`.
    #[cfg(not(unix))]
    assert_eq!(
        in_effect(&config, &registry, &executor, &statement, &key).await,
        Some(false)
    );

    // The type half, on every platform. This is the assertion the metadata read took with
    // it when it was gated for the mode's sake: a file where a directory is declared is not
    // converged, and neither is a directory that is not there at all.
    std::fs::remove_dir(&path).unwrap();
    std::fs::write(&path, "not a directory").unwrap();
    assert_eq!(
        in_effect(&config, &registry, &executor, &statement, &key).await,
        Some(false),
        "a file where a `dir:` is declared is drift"
    );
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        in_effect(&config, &registry, &executor, &statement, &key).await,
        Some(false),
        "a `dir:` that is not on disk at all is drift, and on Windows this was `the machine \
         matches your files`"
    );
}
