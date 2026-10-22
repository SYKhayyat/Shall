use super::*;

#[test]
fn download_only_is_read_on_a_download_backend_and_refused_elsewhere() {
    // D3b: fetch-without-install is a distinction only a downloading backend can draw.
    assert_eq!(
        options_of("github:sharkdp/fd@download_only").one("download_only"),
        Some("true")
    );
    assert_eq!(
        options_of("appimage:https://host/x.AppImage@download_only").one("download_only"),
        Some("true")
    );
    let err = p("apt:curl@download_only").unwrap_err();
    assert!(format!("{}", err).contains("not an option on `apt`"));
}
