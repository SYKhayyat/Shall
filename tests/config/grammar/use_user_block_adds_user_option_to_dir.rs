use super::*;

#[test]
fn use_user_block_adds_user_option_to_dir() {
    let out = stmts("use user:bob {\n  dir:~/logs@mode=0755\n}\n");
    assert_eq!(out.len(), 1);
    let Statement::Dir(name, opts) = &out[0] else {
        panic!("expected dir statement")
    };
    assert_eq!(name, "~/logs");
    assert_eq!(opts.one("user"), Some("bob"));
}
