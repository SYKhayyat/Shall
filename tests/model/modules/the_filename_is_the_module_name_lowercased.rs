use super::*;

#[test]
fn the_filename_is_the_module_name_lowercased() {
    // II.3: `Editors.txt` -> module `editors`. A filename can never mint a profile.
    let f = fixture(&[("editors.txt", "apt:neovim\n")]);
    assert!(expand_module(&f, "Editors").is_ok());
}
