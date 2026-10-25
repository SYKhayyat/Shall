use super::*;

/// A section present but silent about a number is nought, not a parse failure: `check` may
/// grow sections, and a fleet that refuses a document it does not fully recognise is a
/// fleet that stops working the next time `check` gains a line.
#[test]
fn a_missing_count_is_nought() {
    let r = parse_check(r#"[{"section":"drift","ok":true,"counts":{}}]"#).unwrap();
    assert_eq!((r.to_install, r.to_remove, r.unmanaged), (0, 0, 0));
    assert!(!r.drifted);
}
