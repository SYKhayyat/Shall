use super::*;

#[test]
fn the_suffix_match_is_case_insensitive_and_repeats() {
    assert_eq!(strip_archive_suffixes("Tool-1.0.ZIP"), "Tool-1.0");
    assert_eq!(strip_archive_suffixes("tool.tar.gz"), "tool");
}
