use super::*;

#[test]
fn a_bare_string_is_a_command() {
    assert_eq!(
        Probe::parse("systemctl is-active nginx"),
        Some(Probe::Command("systemctl is-active nginx".into()))
    );
    assert_eq!(
        Probe::parse("cmd:true"),
        Some(Probe::Command("true".into()))
    );
}
