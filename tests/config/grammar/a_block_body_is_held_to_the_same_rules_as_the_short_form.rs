use super::*;

/// The block form used to be a way around every rule in II.2's table: the header was
/// validated, the body was merged in afterwards unchecked. Each of these passed clean.
#[test]
fn a_block_body_is_held_to_the_same_rules_as_the_short_form() {
    for (short, block) in [
        (
            "apt:nginx@requires=libfoo",
            "apt:nginx {\n  requires = libfoo\n}",
        ),
        (
            "apt:jq@hold@version=1.6",
            "apt:jq@hold {\n  version = 1.6\n}",
        ),
        ("apt:nginx@colour=blue", "apt:nginx {\n  colour = blue\n}"),
        ("apt:nginx@lease=2h", "apt:nginx {\n  lease = 2h\n}"),
        ("apt:curl@formats=deb", "apt:curl {\n  formats = deb\n}"),
        ("apt:curl@expires=2h", "apt:curl {\n  expires = 2h\n}"),
    ] {
        assert!(
            doc(&format!("{}\n", short)).is_err(),
            "short form allowed `{}`",
            short
        );
        assert!(
            doc(&format!("{}\n", block)).is_err(),
            "block form allowed `{}`",
            block
        );
    }
}
