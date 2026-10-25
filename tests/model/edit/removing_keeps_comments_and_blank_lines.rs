use super::*;

#[test]
fn removing_keeps_comments_and_blank_lines() {
    // Shall must not rewrite your files beyond the line it was asked to remove.
    let f = fx(&[(
        "modules/dev.txt",
        "# my tools\n\napt:curl   # needed for work\napt:jq\n",
    )]);
    let file = f.layout.module_file(&ModuleName::literal("dev"));
    editor(&f).remove_from(&[file], "jq").unwrap();
    let body = read(&f, "modules/dev.txt");
    assert!(body.contains("# my tools"), "{}", body);
    assert!(body.contains("apt:curl   # needed for work"), "{}", body);
    assert!(!body.contains("apt:jq"));
}
