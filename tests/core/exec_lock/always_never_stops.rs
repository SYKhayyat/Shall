use super::*;

#[test]
fn always_never_stops() {
    let c = Ceiling::read(Some("always"));
    assert!(c.permits(0));
    assert!(c.permits(9_999));
}
