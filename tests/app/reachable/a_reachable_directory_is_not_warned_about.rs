use super::*;

/// A directory that IS on PATH produces no warning — otherwise every install on a
/// correctly configured machine nags.
#[test]
fn a_reachable_directory_is_not_warned_about() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("bin");
    std::fs::create_dir_all(&dir).unwrap();
    assert!(!is_on_path(&dir), "a fresh temp dir cannot be on PATH");

    let old = std::env::var_os("PATH");
    std::env::set_var("PATH", &dir);
    let seen = is_on_path(&dir);
    match old {
        Some(p) => std::env::set_var("PATH", p),
        None => std::env::remove_var("PATH"),
    }
    assert!(seen, "a directory that IS on PATH was reported as missing");
}
