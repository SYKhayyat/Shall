use super::*;

#[test]
fn until_is_the_mirror_of_expires() {
    // `absent:apt:steam@until=...` — absent now, present after.
    let o = opts(&[("until", "2026-07-20T00:00")]);
    assert_eq!(dating_of(&o, at("2026-07-19")), Dating::Counting);
    assert_eq!(dating_of(&o, at("2026-07-21")), Dating::Lapsed);
}
