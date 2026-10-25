use super::*;

/// Cutting at the first `#` anywhere made the block form the only way to write a value
/// containing one, and did it without saying a word.
#[test]
fn a_hash_inside_a_value_is_data_and_a_hash_after_a_space_is_a_comment() {
    assert_eq!(strip_comment("apt:curl   # why"), "apt:curl   ");
    assert_eq!(strip_comment("# whole line"), "");
    assert_eq!(
        strip_comment("link:/etc/x@content=#!/bin/sh"),
        "link:/etc/x@content=#!/bin/sh"
    );
}
