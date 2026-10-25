use super::*;

#[test]
fn a_numeric_ceiling_counts() {
    let c = Ceiling::read(Some("3"));
    assert!(c.permits(2));
    assert!(!c.permits(3));
}
