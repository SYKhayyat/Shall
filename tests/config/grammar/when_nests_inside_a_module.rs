use super::*;

#[test]
fn when_nests_inside_a_module() {
    let out = stmts("module fancy {\n  when os == linux {\n    apt:htop\n  }\n}\n");
    assert_eq!(out.len(), 1);
}
