use super::*;

/// A model with all of `J5` in it. Every one of these is a NixOS attribute a user could
/// have written by hand, which is the point: `service:` and `firewall:` are system
/// configuration on this OS, so Shall writes them where NixOS reads them rather than
/// issuing `systemctl` and `ufw` calls the next generation forgets.
#[test]
fn services_and_ports_are_rendered_where_nixos_reads_them() {
    let out = render(&Module {
        packages: set(&["jq"]),
        services: [("nginx".to_string(), true), ("sshd".to_string(), false)]
            .into_iter()
            .collect(),
        tcp_ports: [22u16, 443].into_iter().collect(),
        udp_ports: [53u16].into_iter().collect(),
        firewall: Some(true),
        config_text: Vec::new(),
    })
    .expect("renders");
    assert!(out.contains("services.nginx.enable = true;"), "{out}");
    // A stopped service is `false`, not an omission — see `Module::services`.
    assert!(out.contains("services.sshd.enable = false;"), "{out}");
    assert!(out.contains("networking.firewall.enable = true;"), "{out}");
    assert!(
        out.contains("networking.firewall.allowedTCPPorts = [ 22 443 ];"),
        "{out}"
    );
    assert!(
        out.contains("networking.firewall.allowedUDPPorts = [ 53 ];"),
        "{out}"
    );
}
