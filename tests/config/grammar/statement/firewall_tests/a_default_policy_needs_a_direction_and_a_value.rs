use super::*;

/// N4: the default policy is declarable, and it must say which way it goes — it is the most
/// consequential line in a firewall, so a silent one is the worst case.
#[test]
fn a_default_policy_needs_a_direction_and_a_value() {
    assert!(pv("firewall:default/incoming@value=deny").is_ok());
    assert!(pv("firewall:default/outgoing@value=allow").is_ok());

    let err = pv("firewall:default/incoming").unwrap_err();
    assert!(err.what.contains("needs a policy"), "{}", err);

    assert!(pv("firewall:default/sideways@value=deny").is_err());
    assert!(pv("firewall:default/incoming@value=maybe").is_err());
}
