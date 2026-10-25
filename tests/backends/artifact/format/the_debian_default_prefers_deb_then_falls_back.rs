use super::*;

#[test]
fn the_debian_default_prefers_deb_then_falls_back() {
    let order = FormatOrder::detected_default("linux", Some("debian"));
    assert_eq!(
        order.as_slice(),
        &[
            Format::Deb,
            Format::AppImage,
            Format::Tarball,
            Format::Binary
        ]
    );
}
