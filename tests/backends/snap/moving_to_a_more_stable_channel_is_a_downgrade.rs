use super::*;

#[test]
fn moving_to_a_more_stable_channel_is_a_downgrade() {
    // D13: edge -> stable gives an older build, so the guard/plan should see it.
    assert!(is_channel_downgrade("edge", "stable"));
    assert!(is_channel_downgrade("latest/beta", "latest/candidate"));
    // stable -> edge is newer, not a downgrade.
    assert!(!is_channel_downgrade("stable", "edge"));
    // Same channel, whichever way it is spelled, is not a downgrade.
    assert!(!is_channel_downgrade("latest/stable", "stable"));
    // A channel we do not understand is never called a downgrade.
    assert!(!is_channel_downgrade("weird", "stable"));
}
