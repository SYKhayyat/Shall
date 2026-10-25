use super::*;

#[test]
fn an_empty_probe_is_nothing() {
    assert_eq!(Probe::parse(""), None);
    assert_eq!(Probe::parse("   "), None);
    assert_eq!(Probe::parse("cmd:  "), None);
}
