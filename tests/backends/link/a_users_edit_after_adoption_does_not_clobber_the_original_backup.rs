use super::*;

#[tokio::test]
async fn a_users_edit_after_adoption_does_not_clobber_the_original_backup() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("app.conf");
    tokio::fs::write(&target, "PRISTINE ORIGINAL")
        .await
        .unwrap();
    let inst = installer();

    inst.install(&[inline_spec(&target, "v1")], false)
        .await
        .unwrap();
    // Simulate the user hand-editing the managed file, then a later sync with new content.
    tokio::fs::write(&target, "user tweak").await.unwrap();
    inst.install(&[inline_spec(&target, "v2")], false)
        .await
        .unwrap();

    assert_eq!(tokio::fs::read_to_string(&target).await.unwrap(), "v2");
    // The backup still holds the true pre-Shall original, not the interim edit.
    let backup = backup_path(&target);
    assert_eq!(
        tokio::fs::read_to_string(&backup).await.unwrap(),
        "PRISTINE ORIGINAL"
    );
}
