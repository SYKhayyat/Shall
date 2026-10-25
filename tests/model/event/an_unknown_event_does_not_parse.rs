use super::*;

/// A misspelled event must not parse into a real one. A hook that silently never fires is
/// the worst outcome for something whose job is to tell you.
#[test]
fn an_unknown_event_does_not_parse() {
    assert_eq!(Event::parse("after-sync"), None);
    assert_eq!(Event::parse("AFTER_SYNC"), None);
    assert_eq!(Event::parse(""), None);
    assert_eq!(Event::parse("on_drifts"), None);
}
