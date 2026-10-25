use super::*;

#[tokio::test]
async fn teardown_restores_a_backed_up_directory_tree() {
    let dir = tempdir().unwrap();
    let target = dir.path().join(".config").join("nvim");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("placed-by-shall"), b"managed\n").unwrap();

    let backup = backup_path(&target);
    std::fs::create_dir_all(backup.join("lua")).unwrap();
    std::fs::write(backup.join("init.lua"), b"original\n").unwrap();
    std::fs::write(backup.join("lua").join("m.lua"), b"maps\n").unwrap();

    installer()
        .remove(
            &[target.to_string_lossy().to_string()],
            false,
            removal_token().await,
        )
        .await
        .expect("a backup proves what was there before; restore it");

    assert_eq!(
        std::fs::read(target.join("init.lua")).unwrap(),
        b"original\n"
    );
    assert_eq!(
        std::fs::read(target.join("lua").join("m.lua")).unwrap(),
        b"maps\n"
    );
    assert!(
        !backup.exists(),
        "the restore consumes the backup, as it does for files"
    );
    assert!(
        !target.join("placed-by-shall").exists(),
        "what Shall placed does not outlive its declaration"
    );
}
