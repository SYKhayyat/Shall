use super::*;

#[test]
fn a_trailing_comment_is_stripped_from_a_statement() {
    let out = stmts("apt:curl    # we need this\n");
    let Statement::Package(d) = &out[0] else {
        panic!()
    };
    assert_eq!(d.selector.as_str(), "curl");
}
