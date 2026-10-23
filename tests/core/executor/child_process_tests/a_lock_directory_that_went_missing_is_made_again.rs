use super::*;

/// **The "already created this directory" memo is an optimisation, not a fact.**
///
/// `LOCK_DIRS` records that a lock directory was created once per process, and `create(true)`
/// creates the lock *file*, never its parent — so a directory that goes away after the first
/// exclusive command fails every one after it with a bare *"cannot find the path specified"*,
/// naming neither the directory nor that it vanished.
///
/// Found as a Windows CI failure: the data directory is a process-global setting, the suite
/// points it at temporary directories, and one test deleted a directory another still held
/// the memo for. The arrangement is a test's; the mechanism is not — a `/tmp` reaper does the
/// same thing to a long `sync`.
#[test]
fn a_lock_directory_that_went_missing_is_made_again() {
    let root = tmpdir("execgone");
    let lock_dir = root.join("exec-locks");
    drop(CommandExecutor::open_lock_at(&lock_dir, "brew").unwrap());
    assert!(lock_dir.exists());

    // The memo now says this directory exists. Take it away underneath the memo.
    std::fs::remove_dir_all(&lock_dir).unwrap();
    assert!(!lock_dir.exists());

    drop(
        CommandExecutor::open_lock_at(&lock_dir, "brew")
            .expect("a lock directory that vanished is remade, not reported as a bare io error"),
    );
    assert!(lock_dir.join("brew.lock").exists());
}
