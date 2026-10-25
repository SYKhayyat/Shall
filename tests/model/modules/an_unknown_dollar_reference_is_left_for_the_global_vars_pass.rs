use super::*;

#[test]
fn an_unknown_dollar_reference_is_left_for_the_global_vars_pass() {
    // A param scope substitutes only its own names; `$notaparam` survives verbatim so the
    // later global `vars` pass resolves it (or names it as a typo). It must not error here.
    let f = fixture(&[(
        "m.txt",
        "param user\nlink:./x@target=/home/$user/$notaparam\n",
    )]);
    let stmts = expand_with_args(&f, "m", &[("user", "a")]).unwrap();
    assert_eq!(link_target(&stmts[0]), Some("/home/a/$notaparam"));
}
