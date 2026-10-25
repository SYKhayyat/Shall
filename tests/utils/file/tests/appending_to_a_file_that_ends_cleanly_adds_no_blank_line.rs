use super::*;

#[test]
fn appending_to_a_file_that_ends_cleanly_adds_no_blank_line() {
    let dir = TempDir::new().unwrap();
    let f = dir.path().join("list");
    std::fs::write(
        &f, b"one
",
    )
    .unwrap();
    append_line(&f, "two").unwrap();
    assert_eq!(
        std::fs::read_to_string(&f).unwrap(),
        "one
two
"
    );
}
