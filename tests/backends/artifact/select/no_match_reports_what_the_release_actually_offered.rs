use super::*;

#[test]
fn no_match_reports_what_the_release_actually_offered() {
    let formats = FormatOrder::new(vec![Format::Deb, Format::AppImage]);
    let platform = linux64();
    let err = select(
        &request(&formats, &platform, None),
        &assets(&[
            "fd-v10.2.0-x86_64-unknown-linux-gnu.tar.gz",
            "fd_10.2.0_arm64.deb",
        ]),
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("sharkdp/fd"));
    assert!(msg.contains("deb, appimage"));
    assert!(msg.contains("fd-v10.2.0-x86_64-unknown-linux-gnu.tar.gz"));
    assert!(msg.contains("format not in your list"));
    assert!(msg.contains("fd_10.2.0_arm64.deb"));
    assert!(msg.contains("not for this machine"));
}
