use super::*;

#[test]
fn a_plain_use_of_a_parameterless_module_is_unchanged() {
    // The common case: no params, no args, nothing substituted.
    let f = fixture(&[("editors.txt", "apt:neovim\napt:vim\n")]);
    assert_eq!(expand_module(&f, "editors").unwrap().len(), 2);
}
