use super::*;

#[test]
fn individual_user_and_owner_options_are_validated() {
    let out = stmts("link:./vimrc@target=~/.vimrc,user=alice,owner=alice\n");
    let Statement::Link(_, opts) = &out[0] else {
        panic!("expected link statement")
    };
    assert_eq!(opts.one("user"), Some("alice"));
    assert_eq!(opts.one("owner"), Some("alice"));
}
