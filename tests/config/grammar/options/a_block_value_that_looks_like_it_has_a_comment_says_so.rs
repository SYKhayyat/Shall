use super::*;

#[test]
fn a_block_value_that_looks_like_it_has_a_comment_says_so() {
    // V.9: the value is verbatim, so ` # ` is data. Someone who meant a comment would
    // otherwise get a silently wrong value.
    let err = parse_block_line(&o(), "version = 1.6 # my pin").unwrap_err();
    let hint = err.hint.unwrap();
    assert!(hint.contains("verbatim"), "{}", hint);
    assert!(hint.contains("own line"), "{}", hint);
}
