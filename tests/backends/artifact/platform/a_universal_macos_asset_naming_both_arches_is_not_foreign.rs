use super::*;

#[test]
fn a_universal_macos_asset_naming_both_arches_is_not_foreign() {
    let mac = Platform::new("macos", "aarch64");
    assert!(mac.accepts("tool-macos-universal.dmg"));
    assert!(mac.accepts("tool-macos-x86_64-arm64.dmg"));
}
