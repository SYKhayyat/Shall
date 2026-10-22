use super::*;

/// A declared port is open — that is what declaring it means. `@value=` on one would be a
/// second way to say the same thing, and `firewall:22/tcp @value=deny` reads as both.
#[test]
fn a_port_rule_refuses_a_value() {
    let err = pv("firewall:22/tcp@value=deny").unwrap_err();
    assert!(err.what.contains("takes no `value`"), "{}", err);
    assert!(err.to_string().contains("delete the line"), "{}", err);
}
