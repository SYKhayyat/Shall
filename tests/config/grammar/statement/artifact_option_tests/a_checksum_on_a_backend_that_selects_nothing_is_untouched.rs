use super::*;

#[test]
fn a_checksum_on_a_backend_that_selects_nothing_is_untouched() {
    // `appimage:` already names one file — the backend name is the format — so there is
    // nothing to pin, and demanding `@formats=` there would be unanswerable.
    assert!(p("appimage:https://example.com/tool.AppImage@sha256=abc").is_ok());
}
