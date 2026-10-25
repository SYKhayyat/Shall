use super::*;

/// `none` is how ZFS spells "no quota" and it must not read as a size — a declaration
/// compared against a parsed `none` would look satisfied by a word.
#[test]
fn the_words_a_tool_uses_for_no_limit_are_not_sizes() {
    for word in ["none", "-", "unlimited", "0B?"] {
        assert_eq!(parse_size(word), None, "{}", word);
    }
    // `0` is a number and ZFS's `-p` prints it for "no quota" — so it parses, and reading it
    // is the caller's job, not the parser's.
    assert_eq!(parse_size("0"), Some(0));
}
