use super::*;

#[test]
fn use_user_block_rejects_nested_blocks() {
    let err = doc("use user:alice {\n  module fancy {\n    apt:neovim\n  }\n}\n").unwrap_err();
    assert!(err.to_string().contains("cannot contain blocks"), "{}", err);
}
