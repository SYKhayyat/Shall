use super::*;

#[test]
fn teleport_of_an_undeclared_package_changes_nothing() {
    let f = fx(&[
        ("active", "Work\n"),
        ("profiles/Work", "use tools\n"),
        ("modules/tools.txt", "apt:jq\n"),
    ]);
    let files = vec![f.layout.modules_dir().join("tools.txt")];
    let edits = editor(&f)
        .retarget_backend(&files, "ripgrep", "apt")
        .unwrap();
    assert!(edits.is_empty());
    assert_eq!(read(&f, "modules/tools.txt"), "apt:jq\n");
}
