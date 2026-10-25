use super::*;

#[tokio::test]
async fn a_dry_run_unplace_removes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("shall-packages.nix");
    std::fs::write(&dest, "PREVIOUS").unwrap();
    let (core, _vfs, _mock) = dry_core();

    core.unplace(&dest).await.expect("unplace succeeds");
    assert_eq!(
        std::fs::read_to_string(&dest).unwrap(),
        "PREVIOUS",
        "a dry run does not delete for real"
    );
}
