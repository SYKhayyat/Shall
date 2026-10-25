use super::*;

#[test]
fn a_named_source_that_is_absent_is_an_error() {
    let tmp = tmp();
    let dir = tmp.path();
    std::fs::write(dir.join("vars"), "role = desktop\n").unwrap();
    let err = select(dir, &Some("vars.py".into())).unwrap_err();
    assert!(err.what.contains("not in the repo"), "{}", err);
}
