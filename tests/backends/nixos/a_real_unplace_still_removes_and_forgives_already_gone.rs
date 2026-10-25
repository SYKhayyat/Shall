use super::*;

#[tokio::test]
async fn a_real_unplace_still_removes_and_forgives_already_gone() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("shall-packages.nix");
    std::fs::write(&dest, "MANAGED").unwrap();

    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let exec =
        CommandExecutor::with_layer(false, false, mock, vfs.clone(), Arc::new(DashMap::new()));
    let core = NixosBackendCore::new(
        exec,
        std::env::temp_dir().join("shall-nixos-test-cfg"),
        true,
    );

    core.unplace(&dest).await.expect("removes what it placed");
    assert!(!dest.exists(), "the real path removes for real");
    // Already gone is done — that is what makes teardown idempotent.
    core.unplace(&dest).await.expect("second remove is a no-op");
}
