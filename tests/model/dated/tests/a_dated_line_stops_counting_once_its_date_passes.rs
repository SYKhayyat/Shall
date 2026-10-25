use super::*;

#[test]
fn a_dated_line_stops_counting_once_its_date_passes() {
    // II.7 rule 6. It stops having an opinion; it does not acquire the opposite one.
    let o = opts(&[("expires", "2026-07-17T14:00")]);
    assert_eq!(dating_of(&o, at("2026-07-17T14:01")), Dating::Lapsed);
    assert!(!dating_of(&o, at("2026-07-17T14:01")).counts());
}
