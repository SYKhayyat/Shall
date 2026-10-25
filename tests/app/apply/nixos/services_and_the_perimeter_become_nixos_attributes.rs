use super::*;

/// The whole of `J5`'s fourth ruling in one projection: services and the perimeter reach the
/// attributes NixOS reads.
#[test]
fn services_and_the_perimeter_become_nixos_attributes() {
    let d = declared(&state(vec![
        Statement::Service("nginx".into(), Options::default()),
        Statement::Service("telnet".into(), opts(&[("status", "stopped")])),
        Statement::Firewall("22/tcp".into(), Options::default()),
        Statement::Firewall("53/udp".into(), Options::default()),
        Statement::Firewall("default/incoming".into(), opts(&[("value", "deny")])),
    ]))
    .expect("a state every attribute can express");

    assert_eq!(d.services.get("nginx"), Some(&true));
    assert_eq!(d.services.get("telnet"), Some(&false));
    assert_eq!(d.tcp_ports, [22u16].into_iter().collect());
    assert_eq!(d.udp_ports, [53u16].into_iter().collect());
    assert_eq!(d.firewall, Some(true));
    assert!(d.any);
}
