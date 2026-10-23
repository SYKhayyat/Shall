use super::*;

/// The header carries no `<`, so it is not a row — and a name that matched nothing would
/// be invisible rather than loud.
#[test]
fn the_header_is_not_a_package() {
    assert!(!parse_apk_outdated(APK, "apk")
        .iter()
        .any(|p| p.name.contains("Installed")));
}
