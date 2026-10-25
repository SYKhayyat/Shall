use super::*;

/// An `events:` qualifier names the event **or** the origin, because U15 approves those
/// separately and both are therefore real things to address.
#[test]
fn an_event_qualifier_matches_the_event_or_where_it_is_declared() {
    let by_event = sel("events:before_sync");
    assert!(by_event.admits(LockKind::Events, "event:before_sync@repo", None));
    assert!(!by_event.admits(LockKind::Events, "event:after_sync@repo", None));

    let by_origin = sel("events:preferences");
    assert!(by_origin.admits(LockKind::Events, "event:before_sync@preferences", None));
    assert!(!by_origin.admits(LockKind::Events, "event:before_sync@repo", None));
}
