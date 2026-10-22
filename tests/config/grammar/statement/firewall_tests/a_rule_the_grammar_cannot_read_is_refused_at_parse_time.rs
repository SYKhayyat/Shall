use super::*;

#[test]
fn a_rule_the_grammar_cannot_read_is_refused_at_parse_time() {
    for bad in [
        "firewall:22",
        "firewall:http/tcp",
        "firewall:22/sctp",
        "firewall:0/tcp",
    ] {
        assert!(pv(bad).is_err(), "{} was accepted", bad);
    }
}
