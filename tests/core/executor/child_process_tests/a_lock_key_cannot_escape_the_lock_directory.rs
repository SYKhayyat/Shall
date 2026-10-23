use super::*;

/// A key is a backend name from a config file; one carrying a separator would otherwise
/// pick the directory the lock file lands in.
#[test]
fn a_lock_key_cannot_escape_the_lock_directory() {
    let root = tmpdir("execkey");
    let lock_dir = root.join("exec-locks");
    assert!(CommandExecutor::open_lock_at(&lock_dir, "../../evil").is_ok());
    let landed: Vec<String> = std::fs::read_dir(&lock_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(landed.len(), 1, "{:?}", landed);
    assert!(!landed[0].contains('.') || landed[0].ends_with(".lock"));
    assert!(landed[0].ends_with("evil.lock"), "{:?}", landed);
    assert!(!root.parent().unwrap().join("evil.lock").exists());
}
