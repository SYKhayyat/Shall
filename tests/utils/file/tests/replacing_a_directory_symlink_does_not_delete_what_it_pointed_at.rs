use super::*;

/// **The shape `PLAN.md` #87 is about: a *directory* symlink where the file goes.**
/// `symlink_metadata` does not follow the link, so this path answered "not a directory" and the
/// old arm sent it to `remove_file` — fine on Unix, `Access is denied` on Windows. On this host the
/// old arm worked, so **this test characterises the shape rather than watching the fix fail**;
/// what it does hold is the half that must not regress on either platform, which is that replacing
/// the link leaves what the link pointed at alone.
#[tokio::test]
async fn replacing_a_directory_symlink_does_not_delete_what_it_pointed_at() {
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("real");
    std::fs::create_dir_all(&real).unwrap();
    std::fs::write(real.join("keep-me"), b"x").unwrap();
    let dest = dir.path().join("dest");
    #[cfg(windows)]
    let made = std::os::windows::fs::symlink_dir(&real, &dest).is_ok();
    #[cfg(not(windows))]
    let made = std::os::unix::fs::symlink(&real, &dest).is_ok();
    if !made {
        // Unprivileged Windows without developer mode cannot make one. CI's Windows job can.
        return;
    }
    let src = dir.path().join("src");
    std::fs::write(&src, b"the new contents").unwrap();

    copy_over(&src, &dest)
        .await
        .expect("a symlink in the way is replaceable");

    assert_eq!(
        std::fs::read(&dest).unwrap(),
        b"the new contents",
        "the copy did not land on the symlink's own path"
    );
    assert!(
        real.join("keep-me").exists(),
        "replacing the link deleted what it pointed at — the tree behind a directory symlink is \
     not what was in the way"
    );
}
