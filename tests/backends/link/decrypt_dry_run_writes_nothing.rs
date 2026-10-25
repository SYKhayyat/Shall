use super::*;

#[tokio::test]
async fn decrypt_dry_run_writes_nothing() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("token.age");
    tokio::fs::write(&source, "ENCRYPTED").await.unwrap();
    let target = dir.path().join("token");

    let exec = CommandExecutor::new(true, false);
    let core = Arc::new(LinkBackendCore::new(exec, Arc::new(Config::default())));
    let inst = LinkInstallable { core };
    inst.install(&[decrypt_spec(&source, &target, "age")], false)
        .await
        .unwrap();
    assert!(
        !tokio::fs::try_exists(&target).await.unwrap(),
        "dry-run must not decrypt or write the secret"
    );
}
