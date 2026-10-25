use super::*;

/// The listing is TAB-separated and most flathub apps carry no version, so flatpak emits an
/// empty *middle* field. Split on whitespace — which is what the shared list parser did —
/// and `stable` is read as GIMP's version while its branch goes unread entirely.
#[test]
fn an_empty_version_column_does_not_slide_the_branch_into_it() {
    let out = "org.gimp.GIMP\t\tstable\norg.blender.Blender\t4.0\tbeta\n";
    let pkgs = parse_flatpak_list(out).expect("this fixture parses");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "org.gimp.GIMP");
    assert_eq!(pkgs[0].version, None);
    assert_eq!(
        pkgs[0].properties.get("channel").map(String::as_str),
        Some("stable")
    );
    assert_eq!(pkgs[1].version.as_deref(), Some("4.0"));
    assert_eq!(
        pkgs[1].properties.get("channel").map(String::as_str),
        Some("beta")
    );
}
