use super::*;

#[test]
fn a_dated_line_counts_until_its_date() {
    let o = opts(&[("expires", "2026-07-17T14:00")]);
    assert_eq!(dating_of(&o, at("2026-07-17T13:59")), Dating::Counting);
    assert!(dating_of(&o, at("2026-07-17T13:59")).counts());
}
