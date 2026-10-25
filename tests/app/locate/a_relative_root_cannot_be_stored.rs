use super::*;

#[test]
fn a_relative_root_cannot_be_stored() {
    let err = set_root(Path::new("relative/path")).unwrap_err();
    assert!(err.to_string().contains("absolute"));
}
