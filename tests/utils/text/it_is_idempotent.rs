use super::*;

/// Sanitizing twice is sanitizing once. The parsers that already called it now receive
/// sanitized output from the executor as well, and a second pass must be a no-op rather
/// than something that trims one more layer off a legitimately-indented line.
#[test]
fn it_is_idempotent() {
    for input in [
        "\u{1b}[32mok\u{1b}[0m\r\n",
        "plain",
        "",
        "  leading and trailing  ",
        "a\r\nb\r\nc",
    ] {
        let once = sanitize(input);
        assert_eq!(sanitize(&once), once, "not idempotent for {input:?}");
    }
}
