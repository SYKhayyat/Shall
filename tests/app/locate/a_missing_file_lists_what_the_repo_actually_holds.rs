use super::*;

#[test]
fn a_missing_file_lists_what_the_repo_actually_holds() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("priority"), "apt\n").unwrap();
    let err = resolve_target(dir.path(), Some("prioritty")).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("prioritty"));
    assert!(
        msg.contains("priority"),
        "the error must name the real file"
    );
}
