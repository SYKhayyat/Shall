use super::*;

#[test]
fn the_macos_default_includes_zip() {
    // gh, rclone and starship all ship their macOS build as a .zip; without it those
    // packages resolve to nothing on a Mac (D2).
    let order = FormatOrder::detected_default("macos", None);
    assert!(
        order.as_slice().contains(&Format::Zip),
        "{:?}",
        order.as_slice()
    );
    // dmg and pkg still lead — a native installer beats an archive to unpack.
    assert_eq!(order.as_slice().first(), Some(&Format::Dmg));
}
