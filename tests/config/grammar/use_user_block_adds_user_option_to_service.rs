use super::*;

#[test]
fn use_user_block_adds_user_option_to_service() {
    let out = stmts("use user:charlie {\n  service:nginx@status=running\n}\n");
    assert_eq!(out.len(), 1);
    let Statement::Service(name, opts) = &out[0] else {
        panic!("expected service statement")
    };
    assert_eq!(name, "nginx");
    assert_eq!(opts.one("user"), Some("charlie"));
}
