use super::*;

/// **The defect this shape exists to prevent.** A package install reads the file back,
/// changes the package set and writes it whole — so if it could not see the services and
/// ports, installing one package would silently disable every service Shall declared and
/// close every port it opened. Same class as the rollback that restored one of two files.
#[test]
fn a_package_write_carries_the_services_and_ports_it_knows_nothing_about() {
    let before = Module {
        packages: set(&["jq"]),
        services: [("nginx".to_string(), true)].into_iter().collect(),
        tcp_ports: [22u16].into_iter().collect(),
        udp_ports: [53u16].into_iter().collect(),
        firewall: Some(true),
        config_text: Vec::new(),
    };
    let mut after = read(&render(&before).expect("renders"));
    after.packages.insert("ripgrep".to_string());
    let out = read(&render(&after).expect("renders"));
    assert_eq!(out.services, before.services);
    assert_eq!(out.tcp_ports, before.tcp_ports);
    assert_eq!(out.udp_ports, before.udp_ports);
    assert_eq!(out.firewall, before.firewall);
    assert_eq!(out.packages, set(&["jq", "ripgrep"]));
}
