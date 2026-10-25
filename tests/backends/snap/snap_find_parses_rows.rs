use super::*;

#[test]
fn snap_find_parses_rows() {
    let out = "Name   Version  Publisher  Notes  Summary\n\
               hello  2.10     canonical  -      GNU Hello prints a greeting\n\
               code   1.85     vscode✓    classic Code editing redefined\n";
    let pkgs = parse_snap_find(out);
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "hello");
    assert_eq!(pkgs[0].version.as_deref(), Some("2.10"));
    assert_eq!(
        pkgs[0].properties.get("publisher").map(String::as_str),
        Some("canonical")
    );
    assert!(pkgs[0]
        .properties
        .get("summary")
        .unwrap()
        .contains("greeting"));
}
