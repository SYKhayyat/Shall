use super::*;

/// **The self-waking daemon.** `sync` writes `locks/`, and `watch` is the thing that runs
/// `sync`: if the locks were in the fingerprint, every tick would see the previous tick's
/// writes and reconcile again, for ever.
#[tokio::test]
async fn what_shall_writes_and_git_does_is_not_a_change() {
    let dir = repo().await;
    std::fs::create_dir_all(dir.path().join(".git/objects")).expect("a git dir");
    let before = signature(dir.path()).await;

    std::fs::write(dir.path().join("locks/regex.toml"), "apt:x = [\"y\"]\n").expect("a write");
    std::fs::write(dir.path().join("locks/versions.json"), "{}").expect("another");
    std::fs::write(dir.path().join(".git/HEAD"), "ref: refs/heads/main\n").expect("git churn");
    std::fs::create_dir_all(dir.path().join(".git/objects/ab")).expect("an object dir");
    std::fs::write(dir.path().join(".git/objects/ab/cdef"), "binary-ish").expect("an object");

    assert!(
        changed(&before, &signature(dir.path()).await).is_empty(),
        "watch would wake itself up after every sync"
    );
    assert!(!signature(dir.path()).await.contains_key(".git/HEAD"));
    assert!(!signature(dir.path())
        .await
        .keys()
        .any(|k| k.starts_with("locks/")));
}
