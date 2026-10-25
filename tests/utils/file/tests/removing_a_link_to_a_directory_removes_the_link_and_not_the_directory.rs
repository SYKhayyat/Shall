use super::*;

#[test]
fn removing_a_link_to_a_directory_removes_the_link_and_not_the_directory() {
    // `path.is_dir()` follows the link, so the old body answered yes here and called
    // `remove_dir_all` — which empties the *target* and leaves the link behind. `link:`
    // deploys exactly this shape.
    let dir = TempDir::new().unwrap();
    let real = dir.path().join("real");
    std::fs::create_dir_all(&real).unwrap();
    std::fs::write(real.join("keep-me"), b"x").unwrap();
    let link = dir.path().join("link");
    #[cfg(windows)]
    let made = std::os::windows::fs::symlink_dir(&real, &link).is_ok();
    #[cfg(not(windows))]
    let made = std::os::unix::fs::symlink(&real, &link).is_ok();
    if !made {
        // Unprivileged Windows without developer mode cannot make one; the file arm below
        // still runs, and CI covers this on Linux.
        return;
    }
    force_remove(&link).unwrap();
    assert!(
        real.join("keep-me").exists(),
        "removing the link deleted what it pointed at"
    );
    assert!(
        std::fs::symlink_metadata(&link).is_err(),
        "the link survived"
    );
}
