use super::*;

#[test]
fn an_empty_archive_says_so() {
    let err = find_executable(&[], "x/y", None).unwrap_err();
    assert!(err.to_string().contains("empty"));
}
