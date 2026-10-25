use super::*;

#[test]
fn only_what_is_reached_is_parsed() {
    // II.3. `broken.txt` would fail to parse, but nothing reaches it, so nothing looks.
    let f = fixture(&[
        ("editors.txt", "apt:neovim\n"),
        ("broken.txt", "this is not a statement at all\n"),
    ]);
    assert!(expand_module(&f, "editors").is_ok());
}
