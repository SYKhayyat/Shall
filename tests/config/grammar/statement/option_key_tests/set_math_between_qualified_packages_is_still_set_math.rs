use super::*;

#[test]
fn set_math_between_qualified_packages_is_still_set_math() {
    // The shield must not eat II.4: an operator stands apart from its operands, a name
    // never does. Both spellings of a difference, and the other two operators.
    for line in [
        r"apt:jq \ apt:vim",
        "apt:jq | apt:vim",
        "apt:jq & apt:vim",
        "(Work | gaming) & security",
    ] {
        assert!(
            matches!(parse_line(line).unwrap(), Statement::Expr(_)),
            "{line} must stay a set expression"
        );
    }
}
