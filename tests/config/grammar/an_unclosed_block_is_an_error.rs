use super::*;

#[test]
fn an_unclosed_block_is_an_error() {
    assert!(doc("module fancy {\n  apt:neovim\n").is_err());
    assert!(doc("apt:nginx {\n  after_install = ./x.sh\n").is_err());
}
