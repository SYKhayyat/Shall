use super::*;

#[test]
fn windows_never_falls_back_to_a_linux_artifact() {
    let order = FormatOrder::detected_default("windows", None);
    assert!(!order.as_slice().contains(&Format::AppImage));
    assert!(!order.as_slice().contains(&Format::Deb));
}
