use super::*;

#[test]
fn the_revert_message_says_it_is_restoring_the_snapshot() {
    let msg = reverted_message(&["apt:nginx".to_string()], "snap-1");
    assert!(msg.contains("health check failed"), "{}", msg);
    assert!(msg.contains("restoring the snapshot"), "{}", msg);
    assert!(msg.contains("snap-1"), "{}", msg);
}
