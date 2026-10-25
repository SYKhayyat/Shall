use super::*;

#[test]
fn appending_to_a_file_that_ends_mid_line_does_not_join_onto_it() {
    // A `.gitignore` a user edited by hand is the case: without the newline, adding
    // `*.shall-backup` to a file ending `target` produces `target*.shall-backup`, which
    // ignores neither.
    let dir = TempDir::new().unwrap();
    let f = dir.path().join(".gitignore");
    std::fs::write(&f, b"target").unwrap();
    append_line(&f, "*.shall-backup").unwrap();
    assert_eq!(
        read_lines_filtered(&f).unwrap(),
        vec!["target".to_string(), "*.shall-backup".to_string()]
    );
}
