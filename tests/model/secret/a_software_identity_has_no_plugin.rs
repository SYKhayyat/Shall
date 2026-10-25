use super::*;

/// A software key has no plugin and no touch, so it decrypts the ordinary way — the timeout
/// and the watch-skip are only for hardware.
#[test]
fn a_software_identity_has_no_plugin() {
    assert_eq!(plugin_of("AGE-SECRET-KEY-1QVX9…"), None);
    assert_eq!(plugin_of(""), None);
    assert_eq!(plugin_of("# just a comment\n"), None);
}
