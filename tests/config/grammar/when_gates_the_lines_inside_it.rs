use super::*;

#[test]
fn when_gates_the_lines_inside_it() {
    let out = stmts("when os == linux {\n  apt:htop\n}\n");
    assert_eq!(out.len(), 1);
    let out = stmts("when os == windows {\n  apt:htop\n}\n");
    assert!(out.is_empty());
}
