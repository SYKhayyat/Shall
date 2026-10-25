use super::*;

#[test]
fn the_line_says_so_when_the_declaration_carried_formats() {
    let c = format_choice(
        "github",
        &opts(&[("formats", "deb;tarball"), ("__formats_from", "line")]),
    )
    .unwrap();
    assert_eq!(c, "deb, tarball — set on the line");
}
