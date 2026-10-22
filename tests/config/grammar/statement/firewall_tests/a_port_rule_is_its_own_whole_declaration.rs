use super::*;

#[test]
fn a_port_rule_is_its_own_whole_declaration() {
    let Statement::Firewall(name, opts) = pv("firewall:22/tcp").unwrap() else {
        panic!("not a firewall rule");
    };
    assert_eq!(name, "22/tcp");
    assert!(opts.one("value").is_none());
}
