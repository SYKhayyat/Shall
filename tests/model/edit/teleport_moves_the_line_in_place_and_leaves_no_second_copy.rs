use super::*;

#[test]
fn teleport_moves_the_line_in_place_and_leaves_no_second_copy() {
    let f = fx(&[
        ("active", "Work\n"),
        ("profiles/Work", "use tools\n"),
        ("modules/tools.txt", "cargo:ripgrep\napt:jq\n"),
    ]);
    let files = vec![f.layout.modules_dir().join("tools.txt")];
    let edits = editor(&f)
        .retarget_backend(&files, "ripgrep", "apt")
        .unwrap();
    assert_eq!(edits.len(), 1);
    let body = read(&f, "modules/tools.txt");
    assert!(body.contains("apt:ripgrep"), "{}", body);
    assert!(!body.contains("cargo:ripgrep"), "{}", body);
    // the untouched line stays, and nothing is doubled
    assert_eq!(body.matches("ripgrep").count(), 1, "{}", body);
    assert!(body.contains("apt:jq"), "{}", body);
}
