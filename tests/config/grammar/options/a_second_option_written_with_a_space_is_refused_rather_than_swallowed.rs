use super::*;

/// **B2, in the lexer where it lives.** Options are separated by commas; a space put the
/// second one inside the first one's value, and nothing said so.
#[test]
fn a_second_option_written_with_a_space_is_refused_rather_than_swallowed() {
    for line in [
        "sha256=abc @nosuchkey",
        "version=1.2.3 @sha256=deadbeef",
        "requires=jq @hold",
        "hold @nosuchkey",
        // A tab is a space for this purpose; nobody meant it either.
        "version=1.6\t@hold",
    ] {
        let err = parse_short(&o(), line)
            .expect_err("a space before `@` runs two options together and must be refused");
        let msg = format!("{err}");
        assert!(
            msg.contains("comma") || msg.contains("two options"),
            "the refusal must name the separator, or the user cannot act on it: {msg}"
        );
    }

    // And the specific damage it used to do, asserted as no longer possible: the value was
    // silently wrong and the second option silently absent.
    assert!(parse_short(&o(), "version=1.2.3 @sha256=deadbeef").is_err());
}
