use super::*;

#[test]
fn a_nested_file_resolves() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("modules")).unwrap();
    std::fs::write(dir.path().join("modules/dev.txt"), "apt:jq\n").unwrap();
    let target = resolve_target(dir.path(), Some("modules/dev.txt")).unwrap();
    assert!(target.ends_with("dev.txt"));
}
