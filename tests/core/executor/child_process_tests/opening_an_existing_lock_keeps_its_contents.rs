use super::*;

/// An existing lock file must survive being opened — the contended case is a second
/// process arriving while the first holds it, and truncating it there is how the owner
/// stamp beside it stopped meaning anything.
#[test]
fn opening_an_existing_lock_keeps_its_contents() {
    let root = tmpdir("execkeep");
    let lock_dir = root.join("exec-locks");
    std::fs::create_dir_all(&lock_dir).unwrap();
    std::fs::write(lock_dir.join("dnf.lock"), "held").unwrap();
    drop(CommandExecutor::open_lock_at(&lock_dir, "dnf").unwrap());
    assert_eq!(
        std::fs::read_to_string(lock_dir.join("dnf.lock")).unwrap(),
        "held"
    );
}
