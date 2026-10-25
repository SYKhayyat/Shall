use super::*;

#[test]
fn the_default_ceiling_is_once_per_content() {
    let c = Ceiling::read(None);
    assert!(c.permits(0), "a script that has never run must run");
    assert!(!c.permits(1), "run-once means not twice");
}
