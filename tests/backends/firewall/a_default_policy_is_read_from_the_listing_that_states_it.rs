use super::*;

/// Real `ufw status verbose` output — the row's own `list` command, so this is the output
/// `adopt` will actually see.
///
/// Two claims, and the second is the reason the first is written this way: the default policy
/// is read at all, and the rules are still read from the same output. The row moved from
/// `status numbered` to `status verbose` to get the `Default:` line, and a change of listing
/// command is exactly the kind of thing that silently stops parsing rules.
const UFW_VERBOSE: &str = "Status: active\n\
     Logging: on (low)\n\
     Default: deny (incoming), allow (outgoing), disabled (routed)\n\
     New profiles: skip\n\
     \n\
          To                         Action      From\n\
          --                         ------      ----\n\
     22/tcp                     ALLOW IN    Anywhere\n\
     8080/udp                   ALLOW IN    Anywhere\n\
     22/tcp (v6)                ALLOW IN    Anywhere (v6)\n";

#[test]
fn a_default_policy_is_read_from_the_listing_that_states_it() {
    let ufw = shipped("ufw");
    assert_eq!(
        ufw.parse_defaults(UFW_VERBOSE),
        vec![
            (Direction::Incoming, "deny".to_string()),
            (Direction::Outgoing, "allow".to_string()),
        ],
        "the two directions this model has"
    );

    // `disabled (routed)` is a third pair on the same line, and `disabled` is a policy for a
    // direction `Direction` has no name for. Widening it into incoming or outgoing would write
    // a line nobody measured; the assertion is that nothing was invented for it.
    assert!(
        !ufw.parse_defaults(UFW_VERBOSE)
            .iter()
            .any(|(_, p)| p == "disabled"),
        "a direction with no name here is skipped, not widened"
    );

    // The same output still yields its rules — the pair of questions, one command. Two distinct
    // rules, not three: `22/tcp (v6)` is the same `Rule`, because the model has no IP-version
    // axis and `firewall:22/tcp` opens both. Reading it as a third rule would offer a line that
    // says nothing different from the one already there.
    let rules: Vec<String> = ufw
        .parse_rules(UFW_VERBOSE)
        .iter()
        .map(|r| r.to_string())
        .collect();
    assert_eq!(rules, vec!["22/tcp", "8080/udp"], "{rules:?}");
}

#[test]
fn a_firewall_that_cannot_state_its_default_offers_none() {
    // firewalld's row lists ports and nothing else, and Windows Defender's `show rule` states
    // the policy as one line carrying both directions. Both answer `vec![]`, which `adopt`
    // prints as "this firewall cannot say what its default policy is" — never as "the default
    // is open", which is the reading that gets a machine firewalled into silence.
    for name in ["firewalld", "windows-defender"] {
        let a = shipped(name);
        assert!(
            a.parse_defaults("Default: deny (incoming), allow (outgoing)")
                .is_empty(),
            "{name} must not report a policy it cannot read"
        );
    }
}
