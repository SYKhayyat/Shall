use super::*;

#[test]
fn a_missing_module_names_what_is_available() {
    let f = fixture(&[("editors.txt", "apt:neovim\n")]);
    let err = expand_module(&f, "editrs").unwrap_err();
    assert!(err.what.contains("no module named `editrs`"), "{}", err);
    assert!(err.hint.unwrap().contains("editors"));
}
