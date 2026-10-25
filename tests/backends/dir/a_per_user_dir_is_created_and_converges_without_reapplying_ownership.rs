use super::*;

#[tokio::test]
async fn a_per_user_dir_is_created_and_converges_without_reapplying_ownership() {
    let root = tempdir().unwrap();
    let account = Account::current().unwrap();
    let path = root.path().join("nested").join("state");
    // **The mode is declared only where it can be honoured.** `install` refuses a `mode` on a
    // platform with no mode bits, by name — so a fixture that declares one there is asking
    // for a refusal and then failing on it, which measures the fixture rather than the
    // backend. The per-user half is the subject, and `@user` is real on both platforms.
    let mut declared: Vec<(&str, &str)> = vec![("user", account.name())];
    if cfg!(unix) {
        declared.push(("mode", "0750"));
    }
    let declaration = spec(path.to_str().unwrap(), &declared);
    std::fs::create_dir_all(&path).unwrap();
    let installer = installer();
    installer
        .install(std::slice::from_ref(&declaration), false)
        .await
        .unwrap();
    assert!(path.is_dir());
    assert!(account.owns(&path, true).unwrap());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o7777,
            0o750
        );
    }
    installer.install(&[declaration], false).await.unwrap();
    assert!(path.is_dir());
}
