use super::*;

#[test]
fn a_plugin_identity_is_recognised_by_name() {
    assert_eq!(
        plugin_of("AGE-PLUGIN-YUBIKEY-1QVX…").as_deref(),
        Some("yubikey")
    );
    // age-keygen writes the recipient in a comment above the identity.
    assert_eq!(
        plugin_of("# public key: age1yubikey1…\nAGE-PLUGIN-YUBIKEY-1ABC\n").as_deref(),
        Some("yubikey")
    );
    // A different plugin.
    assert_eq!(plugin_of("age-plugin-tpm-1xyz").as_deref(), Some("tpm"));
}
