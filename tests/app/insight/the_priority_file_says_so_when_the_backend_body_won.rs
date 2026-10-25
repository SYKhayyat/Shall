use super::*;

#[test]
fn the_priority_file_says_so_when_the_backend_body_won() {
    let c = format_choice(
        "github",
        &opts(&[
            ("formats", "appimage;binary"),
            ("__formats_from", "priority (github)"),
        ]),
    )
    .unwrap();
    assert_eq!(c, "appimage, binary — set by `github` in `priority`");
}
