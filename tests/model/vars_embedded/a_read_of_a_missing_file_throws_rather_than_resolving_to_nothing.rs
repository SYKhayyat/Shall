use super::*;

#[test]
fn a_read_of_a_missing_file_throws_rather_than_resolving_to_nothing() {
    let err = run(r#"#{ x: read_file("/no/such/file/anywhere") }"#).unwrap_err();
    assert!(err.what.contains("read_file"), "{}", err);
}
