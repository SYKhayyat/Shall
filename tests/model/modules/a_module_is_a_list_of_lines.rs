use super::*;

#[test]
fn a_module_is_a_list_of_lines() {
    let f = fixture(&[("editors.txt", "apt:neovim\napt:vim\n")]);
    assert_eq!(expand_module(&f, "editors").unwrap().len(), 2);
}
