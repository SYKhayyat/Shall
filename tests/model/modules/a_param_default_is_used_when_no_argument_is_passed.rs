use super::*;

#[test]
fn a_param_default_is_used_when_no_argument_is_passed() {
    let f = fixture(&[(
        "workstation.txt",
        "param dir = /opt\nlink:./x@target=$dir/x\n",
    )]);
    let stmts = expand_with_args(&f, "workstation", &[]).unwrap();
    assert_eq!(link_target(&stmts[0]), Some("/opt/x"));
}
