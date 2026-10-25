use super::*;

/// **The round trip over the whole model**, which is what makes one file safe for two
/// writers. `list` depends on the package half; the `service:`/`firewall:` path depends on
/// every other field surviving a package install that knows nothing about it.
#[test]
fn what_is_rendered_is_what_is_read_back() {
    for want in [
        Module::default(),
        packages(&["curl", "jq", "ripgrep"]),
        Module {
            packages: set(&["jq"]),
            services: [("nginx".to_string(), true), ("sshd".to_string(), false)]
                .into_iter()
                .collect(),
            tcp_ports: [22u16, 443].into_iter().collect(),
            udp_ports: [53u16].into_iter().collect(),
            firewall: Some(true),
            config_text: Vec::new(),
        },
        Module {
            firewall: Some(false),
            ..Module::default()
        },
    ] {
        assert_eq!(
            read(&render(&want).expect("renders")),
            want,
            "{}",
            render(&want).expect("renders")
        );
    }
}
