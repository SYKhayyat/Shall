use super::*;

#[test]
fn channel_is_read_on_snap_and_flatpak() {
    assert_eq!(
        options_of("snap:code@channel=stable").one("channel"),
        Some("stable")
    );
    assert_eq!(
        options_of("flatpak:org.gimp.GIMP@channel=stable").one("channel"),
        Some("stable")
    );
}
