use super::*;

/// A line the module cannot read is refused with a reason, not quietly skipped.
#[test]
fn a_line_that_is_not_a_pattern_says_why() {
    assert!(Pattern::parse("!keep.txt").is_err());
    assert!(Pattern::parse("[ab]*.txt").is_err());
    assert!(Pattern::parse("/").is_err());
}
