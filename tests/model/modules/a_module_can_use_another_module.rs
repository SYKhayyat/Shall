use super::*;

#[test]
fn a_module_can_use_another_module() {
    let f = fixture(&[
        ("dev.txt", "use editors\ncargo:ripgrep\n"),
        ("editors.txt", "apt:neovim\n"),
    ]);
    assert_eq!(expand_module(&f, "dev").unwrap().len(), 2);
}
