use super::*;

/// ...and where it is not, that is a named `None` rather than a silent skip. firewalld has
/// no outgoing target, and pretending otherwise would report a policy nobody set.
#[test]
fn a_direction_a_firewall_cannot_express_is_not_pretended() {
    assert!(shipped("firewalld")
        .default_command(Direction::Outgoing, "deny")
        .is_none());
    assert!(shipped("firewalld")
        .default_command(Direction::Incoming, "DROP")
        .is_some());
}
