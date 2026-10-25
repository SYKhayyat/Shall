use super::*;

#[test]
fn a_declaration_block_keeps_commas_and_equals_in_a_value() {
    let out = stmts("apt:nginx {\n  after_install = ./setup.sh --flag=a,b\n}\n");
    let Statement::Package(d) = &out[0] else {
        panic!()
    };
    assert_eq!(
        d.options.one("after_install"),
        Some("./setup.sh --flag=a,b")
    );
}
