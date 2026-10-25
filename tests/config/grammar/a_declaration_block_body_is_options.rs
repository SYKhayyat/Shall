use super::*;

#[test]
fn a_declaration_block_body_is_options() {
    // The header decides the body kind: `apt:nginx` is not a keyword, so options.
    let out = stmts("apt:nginx {\n  after_install = ./setup.sh\n}\n");
    let Statement::Package(d) = &out[0] else {
        panic!()
    };
    assert_eq!(d.options.one("after_install"), Some("./setup.sh"));
}
