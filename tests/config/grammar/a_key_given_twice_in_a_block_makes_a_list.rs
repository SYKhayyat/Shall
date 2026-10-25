use super::*;

#[test]
fn a_key_given_twice_in_a_block_makes_a_list() {
    let out = stmts("apt:nginx {\n  requires = apt:libfoo\n  requires = apt:libbar\n}\n");
    let Statement::Package(d) = &out[0] else {
        panic!()
    };
    assert_eq!(d.options.all("requires"), ["apt:libfoo", "apt:libbar"]);
}
