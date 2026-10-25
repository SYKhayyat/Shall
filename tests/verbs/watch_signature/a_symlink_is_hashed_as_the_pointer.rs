use super::*;

/// A symlink is the pointer, not what it points at: a change in a directory *outside* the
/// repo is not a change in the repo, and re-pointing one is.
#[cfg(unix)]
#[tokio::test]
async fn a_symlink_is_hashed_as_the_pointer() {
    let dir = repo().await;
    let outside = tempdir().expect("a tempdir");
    let target = outside.path().join("watched.txt");
    std::fs::write(&target, "one").expect("the target");
    std::os::unix::fs::symlink(&target, dir.path().join("modules/linked.txt")).expect("the link");

    let before = signature(dir.path()).await;
    std::fs::write(&target, "two").expect("the target changed outside the repo");
    assert!(
        changed(&before, &signature(dir.path()).await).is_empty(),
        "a file outside the repo changed and watch woke up for it"
    );

    let other = outside.path().join("other.txt");
    std::fs::write(&other, "three").expect("a second target");
    std::fs::remove_file(dir.path().join("modules/linked.txt")).expect("unlink");
    std::os::unix::fs::symlink(&other, dir.path().join("modules/linked.txt")).expect("re-link");
    assert_eq!(
        changed(&before, &signature(dir.path()).await),
        vec!["modules/linked.txt".to_string()],
        "re-pointing a symlink is a change to what the repo declares"
    );
}
