use super::*;

#[tokio::test]
async fn no_backup_created_when_target_absent() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("fresh.conf");
    let inst = installer();
    inst.install(&[inline_spec(&target, "HELLO")], false)
        .await
        .unwrap();

    assert_eq!(tokio::fs::read_to_string(&target).await.unwrap(), "HELLO");
    let backup = backup_path(&target);
    assert!(
        !tokio::fs::try_exists(&backup).await.unwrap(),
        "nothing pre-existed, so no backup should be written"
    );
}
