use super::*;

#[test]
fn use_user_block_adds_user_option_to_link() {
    let out = stmts("use user:alice {\n  link:./vimrc@target=~/vimrc\n}\n");
    assert_eq!(out.len(), 1);
    let Statement::Link(name, opts) = &out[0] else {
        panic!("expected link statement")
    };
    assert_eq!(name, "./vimrc");
    assert_eq!(opts.one("user"), Some("alice"));
}
