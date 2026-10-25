use super::*;

#[test]
fn a_missing_list_is_an_empty_list_and_comments_are_not_entries() {
    let dir = TempDir::new().unwrap();
    let f = dir.path().join("list");
    assert!(read_lines_filtered(&f).unwrap().is_empty());
    std::fs::write(
        &f,
        b"  # a note

  keep\x20\x20
#*.commented-out
",
    )
    .unwrap();
    assert_eq!(read_lines_filtered(&f).unwrap(), vec!["keep".to_string()]);
    assert_eq!(
        filtered_lines(
            "a
# b

c"
        ),
        vec!["a".to_string(), "c".to_string()]
    );
}
