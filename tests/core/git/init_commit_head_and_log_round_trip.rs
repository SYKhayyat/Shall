use super::*;

#[test]
fn init_commit_head_and_log_round_trip() {
    if !git_or_skip() {
        return;
    }
    let tmp = tempdir().unwrap();
    let git = GitManager::new(tmp.path());
    assert!(!git.is_repo());
    git.init().unwrap();
    assert!(git.is_repo());
    assert!(git.head().unwrap().is_none(), "no commits yet");

    std::fs::write(tmp.path().join("local.txt"), "apt:curl\n").unwrap();
    let first = git.commit_all("add curl").unwrap();
    assert!(first.is_some());
    let head = git.head().unwrap().unwrap();
    assert_eq!(head, first.unwrap());

    assert!(git.commit_all("noop").unwrap().is_none());

    let log = git.log(10).unwrap();
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].subject, "add curl");
}
