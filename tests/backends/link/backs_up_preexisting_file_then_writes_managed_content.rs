use super::*;

#[tokio::test]
async fn backs_up_preexisting_file_then_writes_managed_content() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("gitconfig");
    tokio::fs::write(&target, "ORIGINAL USER CONTENT")
        .await
        .unwrap();

    let inst = installer();
    let spec = inline_spec(&target, "MANAGED CONTENT");
    inst.install(std::slice::from_ref(&spec), false)
        .await
        .unwrap();

    // The managed content is in place, and the user's original is preserved.
    assert_eq!(
        tokio::fs::read_to_string(&target).await.unwrap(),
        "MANAGED CONTENT"
    );
    let backup = backup_path(&target);
    assert_eq!(
        tokio::fs::read_to_string(&backup).await.unwrap(),
        "ORIGINAL USER CONTENT"
    );

    // Idempotent: re-applying does not touch the single original backup.
    inst.install(&[spec], false).await.unwrap();
    assert_eq!(
        tokio::fs::read_to_string(&backup).await.unwrap(),
        "ORIGINAL USER CONTENT"
    );
}
