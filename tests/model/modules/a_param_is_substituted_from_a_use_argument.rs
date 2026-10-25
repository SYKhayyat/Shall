use super::*;

#[test]
fn a_param_is_substituted_from_a_use_argument() {
    // U32: the flagship example. `$user` in the module body becomes the passed value, and
    // the `param` line itself never reaches the output.
    let f = fixture(&[(
        "workstation.txt",
        "param user\nlink:./gitconfig@target=/home/$user/.gitconfig\n",
    )]);
    let stmts = expand_with_args(&f, "workstation", &[("user", "shaul")]).unwrap();
    assert_eq!(stmts.len(), 1, "the param line is consumed: {:?}", stmts);
    assert_eq!(link_target(&stmts[0]), Some("/home/shaul/.gitconfig"));
}
