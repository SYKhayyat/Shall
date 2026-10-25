use super::*;

#[test]
fn the_moment_itself_has_passed() {
    let o = opts(&[("expires", "2026-07-17T14:00")]);
    assert_eq!(dating_of(&o, at("2026-07-17T14:00")), Dating::Lapsed);
}
