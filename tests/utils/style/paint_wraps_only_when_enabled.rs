use super::*;

#[test]
fn paint_wraps_only_when_enabled() {
    assert_eq!(paint(false, GREEN, "OK"), "OK");
    assert_eq!(paint(true, GREEN, "OK"), "\u{1b}[32mOK\u{1b}[0m");
}
