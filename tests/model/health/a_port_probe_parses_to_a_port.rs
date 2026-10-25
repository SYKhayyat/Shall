use super::*;

#[test]
fn a_port_probe_parses_to_a_port() {
    assert_eq!(Probe::parse("port:8080"), Some(Probe::Port(8080)));
    assert_eq!(Probe::parse("port: 443 "), Some(Probe::Port(443)));
}
