use super::*;

#[test]
fn an_unknown_family_still_has_a_working_default() {
    let order = FormatOrder::detected_default("linux", Some("alpine"));
    assert_eq!(
        order.as_slice(),
        &[Format::AppImage, Format::Tarball, Format::Binary]
    );
}
