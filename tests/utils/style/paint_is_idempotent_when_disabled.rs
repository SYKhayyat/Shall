use super::*;

#[test]
fn paint_is_idempotent_when_disabled() {
    // Disabled painting never alters the string, so it's safe to always route through it.
    let s = "hello world";
    assert_eq!(paint(false, RED, s), s);
}
