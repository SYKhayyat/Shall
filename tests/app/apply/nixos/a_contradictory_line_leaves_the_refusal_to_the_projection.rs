use super::*;

/// A `service:` line whose declaration is a contradiction returns `None` here rather than
/// panicking — the refusal is `declared`'s to raise, with the origin in hand, and this must
/// not race it to a worse message.
#[test]
fn a_contradictory_line_leaves_the_refusal_to_the_projection() {
    assert!(imperative_remainder(&opts(&[("enabled", "false"), ("status", "running")])).is_none());
}
