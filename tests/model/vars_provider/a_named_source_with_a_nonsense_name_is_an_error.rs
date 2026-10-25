use super::*;

#[test]
fn a_named_source_with_a_nonsense_name_is_an_error() {
    let tmp = tmp();
    let dir = tmp.path();
    let err = select(dir, &Some("something".into())).unwrap_err();
    assert!(err.what.contains("not a variable provider name"), "{}", err);
}
