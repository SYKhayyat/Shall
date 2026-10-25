use super::*;

#[test]
fn a_lone_executable_under_a_different_name_is_accepted() {
    let e = entries(&[("ripgrep-13/rg", true), ("ripgrep-13/README", false)]);
    let found = find_executable(&e, "BurntSushi/ripgrep", None).unwrap();
    assert_eq!(normalise(&found), "ripgrep-13/rg");
}
