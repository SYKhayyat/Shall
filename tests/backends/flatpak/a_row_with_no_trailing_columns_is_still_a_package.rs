use super::*;

/// Bytes off a real flathub listing (`debian:12` container, flatpak 1.14.10): the columns
/// are tab-separated and a trailing empty column is dropped rather than emitted.
#[test]
fn a_row_with_no_trailing_columns_is_still_a_package() {
    let pkgs = parse_flatpak_list("ai.jan.Jan\n").expect("this fixture parses");
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0].name, "ai.jan.Jan");
    assert_eq!(pkgs[0].version, None);
    assert_eq!(pkgs[0].properties.get("channel"), None);
}
