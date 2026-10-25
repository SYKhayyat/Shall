use super::*;

#[test]
fn a_nested_use_receives_substituted_arguments() {
    // `use inner(who=$user)` inside a parameterized module: the outer param is substituted
    // into the inner call's argument before the inner module binds it.
    let f = fixture(&[
        ("outer.txt", "param user\nuse inner(who=$user)\n"),
        ("inner.txt", "param who\nlink:./x@target=/home/$who/x\n"),
    ]);
    let stmts = expand_with_args(&f, "outer", &[("user", "shaul")]).unwrap();
    assert_eq!(link_target(&stmts[0]), Some("/home/shaul/x"));
}
