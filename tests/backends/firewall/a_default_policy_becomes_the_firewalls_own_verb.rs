use super::*;

/// N4: the default policy is expressible where the firewall has one...
#[test]
fn a_default_policy_becomes_the_firewalls_own_verb() {
    let cmd = shipped("ufw")
        .default_command(Direction::Incoming, "deny")
        .expect("ufw sets a default");
    assert_eq!(cmd, vec!["ufw", "default", "deny", "incoming"]);
}
