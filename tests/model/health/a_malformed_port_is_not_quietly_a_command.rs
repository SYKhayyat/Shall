use super::*;

/// A malformed port must not become a command named `port:donkey`. The line says port, so
/// it is a port or it is nothing — silently running it as a shell command would be the
/// worst reading of a typo.
#[test]
fn a_malformed_port_is_not_quietly_a_command() {
    assert_eq!(Probe::parse("port:donkey"), None);
    assert_eq!(Probe::parse("port:99999"), None);
    assert_eq!(Probe::parse("port:"), None);
}
