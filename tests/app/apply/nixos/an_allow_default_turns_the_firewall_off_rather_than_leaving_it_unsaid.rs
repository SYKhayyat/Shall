use super::*;

/// `allow` is the other half of the default policy, and it is the one that would be silently
/// wrong: an unwritten attribute is not "allow", it is NixOS's own default.
#[test]
fn an_allow_default_turns_the_firewall_off_rather_than_leaving_it_unsaid() {
    let d = declared(&state(vec![Statement::Firewall(
        "default/incoming".into(),
        opts(&[("value", "allow")]),
    )]))
    .expect("allow is expressible");
    assert_eq!(d.firewall, Some(false));
}
