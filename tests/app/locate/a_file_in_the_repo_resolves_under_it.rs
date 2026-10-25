use super::*;

#[test]
fn a_file_in_the_repo_resolves_under_it() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("priority"), "apt\n").unwrap();
    let target = resolve_target(dir.path(), Some("priority")).unwrap();
    assert_eq!(target, dir.path().join("priority"));
}
