use super::*;

#[test]
fn an_unreached_broken_module_is_never_parsed() {
    // II.3: Shall only parses what the active profiles reach.
    let f = fx(
        "Work\n",
        &[("Work", "use editors\n")],
        &[
            ("editors.txt", "apt:neovim\n"),
            ("broken.txt", "!!! not a statement !!!\n"),
        ],
    );
    assert!(resolve(&f).is_ok());
}
