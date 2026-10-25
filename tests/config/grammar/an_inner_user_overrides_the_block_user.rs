use super::*;

#[test]
fn an_inner_user_overrides_the_block_user() {
    let out = stmts("use user:alice {\n  link:./vimrc@target=~/vimrc,user=bob\n}\n");
    let Statement::Link(_, options) = &out[0] else {
        panic!()
    };
    assert_eq!(options.one("user"), Some("bob"));
}
