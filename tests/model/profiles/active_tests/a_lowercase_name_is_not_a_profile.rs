use super::*;

#[test]
fn a_lowercase_name_is_not_a_profile() {
    let err = read("editors\n", "any").unwrap_err();
    assert!(err.hint.unwrap().contains("profiles are Capitalized"));
}
