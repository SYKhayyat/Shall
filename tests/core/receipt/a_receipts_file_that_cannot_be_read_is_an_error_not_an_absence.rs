use super::*;

#[test]
fn a_receipts_file_that_cannot_be_read_is_an_error_not_an_absence() {
    let tmp = tempdir().unwrap();
    let dir = tmp.path().join("receipts.jsonl");
    std::fs::create_dir(&dir).unwrap();
    let err = Receipts::at(dir).unwrap_err();
    assert!(
        err.to_string().contains("install receipts"),
        "the error names the file rather than reading as 'no claims': {err}"
    );
}
