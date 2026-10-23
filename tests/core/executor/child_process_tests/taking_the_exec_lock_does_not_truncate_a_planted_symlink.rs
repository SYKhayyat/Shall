#[cfg(unix)]
use super::*;

/// The lock is a shared, guessable name by design — that is what makes it a lock. It must
/// therefore never truncate what it opens, or a symlink planted at that path destroys the
/// file it points at, often as root.
#[cfg(unix)]
#[test]
fn taking_the_exec_lock_does_not_truncate_a_planted_symlink() {
    let root = tmpdir("execlock");
    let canary = root.join("canary");
    std::fs::write(&canary, "must survive").unwrap();
    let lock_dir = root.join("exec-locks");
    std::fs::create_dir_all(&lock_dir).unwrap();
    std::os::unix::fs::symlink(&canary, lock_dir.join("apt.lock")).unwrap();

    drop(CommandExecutor::open_lock_at(&lock_dir, "apt").unwrap());
    assert_eq!(std::fs::read_to_string(&canary).unwrap(), "must survive");
}
