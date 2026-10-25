use super::*;

#[test]
fn a_module_block_must_be_lowercase() {
    // A Capitalized name would mint a profile, and only profiles/ may do that (II.5).
    let err = doc("module Fancy {\n  apt:neovim\n}\n").unwrap_err();
    assert!(err.hint.unwrap().contains("modules are lowercase"));
}
