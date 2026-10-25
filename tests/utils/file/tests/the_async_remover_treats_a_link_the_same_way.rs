use super::*;

#[tokio::test]
async fn the_async_remover_treats_a_link_the_same_way() {
    // `link:` removes user paths through this one, and the two removers disagreeing about
    // what a symlink is would be the whole point of sharing them, lost.
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
        return;
    }
    remove_deployed_path(&link).await.unwrap();
    assert!(real.join("keep-me").exists());
    assert!(std::fs::symlink_metadata(&link).is_err());
}
