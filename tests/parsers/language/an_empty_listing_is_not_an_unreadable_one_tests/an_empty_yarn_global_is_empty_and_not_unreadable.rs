use super::*;

#[test]
fn an_empty_yarn_global_is_empty_and_not_unreadable() {
    // yarn brackets every command with a banner and a footer, so an empty global install
    // still prints two lines — and neither is prose by the general rule.
    let out = "yarn global v1.22.22
Done in 0.67s.
";
    assert_eq!(
        parse_installed("yarn", out).expect("an empty yarn global is empty, not unreadable"),
        Vec::new()
    );
}
