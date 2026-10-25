use super::*;

/// **Ownership is not proven by the absence of a backup.** A user who replaced Shall's
/// managed link with their own directory left nothing for the teardown to recognise, and
/// remove used to delete whatever it found — their tree included. `link:` places links and
/// files only, so a real *directory* at that path with no `.shall-backup` behind it cannot
/// be something Shall put there.
#[tokio::test]
async fn teardown_never_deletes_a_directory_shall_did_not_place() {
    let dir = tempdir().unwrap();
    let target = dir.path().join(".config").join("nvim");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("init.lua"), b"user data\n").unwrap();

    let err = installer()
        .remove(
            &[target.to_string_lossy().to_string()],
            false,
            removal_token().await,
        )
        .await
        .expect_err("a directory with no Shall backup is not provenance to remove it");

    assert!(
        err.to_string().contains("directory"),
        "the refusal says what stood in the way: {err}"
    );
    assert!(
        target.join("init.lua").exists(),
        "the user's tree survives a teardown that cannot prove ownership"
    );
}
