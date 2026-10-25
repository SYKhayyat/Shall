use super::*;

#[test]
fn a_file_that_is_not_toml_names_the_place_it_broke() {
    let s = standing(&[("secret", "this is not toml\n")], "secret");
    assert!(matches!(s, Standing::Malformed(_)), "{s:?}");
    assert!(s.is_wrong());
    assert!(s.detail().is_some_and(|d| d.contains("line")), "{s:?}");
}
