use super::*;

#[test]
fn an_unknown_key_lists_the_real_ones() {
    // II.2's table is the whole list. A key nobody reads is a line that does nothing; a
    // key someone still reads is worse.
    let err = parse_line("apt:jq@colour=blue").unwrap_err();
    assert!(err.what.contains("`@colour` is not an option"), "{}", err);
    let hint = err.hint.unwrap();
    assert!(hint.contains("version"), "{}", hint);
    assert!(hint.contains("requires"), "{}", hint);
}
