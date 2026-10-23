use super::*;

#[test]
fn apk_splits_the_name_the_same_way_the_listing_does() {
    let p = parse_apk_outdated(APK, "apk");
    assert_eq!(p.len(), 3);
    assert_eq!(p[0].name, "apk-tools");
    assert_eq!(p[0].version.as_deref(), Some("3.0.7-r0"));
    assert_eq!(p[2].name, "nodejs");
    assert_eq!(p[2].version.as_deref(), Some("24.18.1-r0"));
}
