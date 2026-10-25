use super::*;

/// The boundary. An `@` is ordinary inside a value and stays legal — banning it outright
/// would refuse a scoped npm package and a pinned module source, which are the two things
/// most likely to be written as an option value.
#[test]
fn an_at_sign_inside_a_value_is_not_two_options() {
    for (line, key, value) in [
        ("requires=@angular/cli", "requires", "@angular/cli"),
        (
            "source=github:owner/repo@v2",
            "source",
            "github:owner/repo@v2",
        ),
        ("version=1.2.3+build@7", "version", "1.2.3+build@7"),
    ] {
        let opts = parse_short(&o(), line)
            .unwrap_or_else(|e| panic!("`{line}` is an ordinary value and was refused: {e}"));
        assert_eq!(opts.one(key), Some(value));
    }

    // A value may still carry a space when no `@` follows it — the rule is about the
    // separator that was mistyped, not about spaces.
    assert_eq!(
        parse_short(&o(), "content=hello world")
            .unwrap()
            .one("content"),
        Some("hello world")
    );
}
