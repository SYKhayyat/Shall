use super::*;

#[test]
fn a_module_block_body_is_lines() {
    let out = stmts("module fancy {\n  apt:neovim\n  cargo:ripgrep\n}\n");
    assert_eq!(out.len(), 2);
}
