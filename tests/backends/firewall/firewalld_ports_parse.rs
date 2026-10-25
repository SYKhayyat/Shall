use super::*;

#[test]
fn firewalld_ports_parse() {
    let rules = shipped("firewalld").parse_rules("22/tcp 443/tcp 53/udp\n");
    assert_eq!(rules.len(), 3, "{:?}", rules);
}
