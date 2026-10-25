use super::*;

/// **The user's own ignore list**, in the shape the module documents: globs, a trailing `/`
/// for a directory, a leading `/` for the root, `#` for a comment, and `**` across
/// directories.
#[tokio::test]
async fn the_ignore_file_is_the_users_own_list() {
    let dir = repo().await;
    std::fs::write(
        dir.path().join(IGNORE_FILE),
        "# not an input\nREADME.md\n/docs/\n**/*.bak\n*.tmp\n",
    )
    .expect("the ignore file");
    std::fs::create_dir_all(dir.path().join("docs")).expect("a docs dir");
    std::fs::write(dir.path().join("README.md"), "hello").expect("a readme");
    std::fs::write(dir.path().join("docs/notes.md"), "notes").expect("a doc");
    std::fs::write(dir.path().join("modules/dev.txt.bak"), "old").expect("a backup");
    std::fs::write(dir.path().join("scratch.tmp"), "wip").expect("a temp file");

    let before = signature(dir.path()).await;
    for name in [
        "README.md",
        "docs/notes.md",
        "modules/dev.txt.bak",
        "scratch.tmp",
    ] {
        assert!(
            !before.contains_key(name),
            "{name} is in the signature and the ignore file names it"
        );
    }
    assert!(
        before.contains_key("modules/dev.txt"),
        "the ignore list ate a manifest"
    );
    assert!(
        before.contains_key(IGNORE_FILE),
        "the ignore file must be watched itself"
    );

    // And an edit to an ignored file is not a change, which is the point of the file.
    std::fs::write(dir.path().join("README.md"), "goodbye").expect("the edit");
    assert!(changed(&before, &signature(dir.path()).await).is_empty());
}
