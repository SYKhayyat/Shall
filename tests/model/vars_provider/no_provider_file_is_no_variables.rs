use super::*;

#[test]
fn no_provider_file_is_no_variables() {
    let tmp = tmp();
    let dir = tmp.path();
    assert!(select(dir, &None).unwrap().is_none());
}
