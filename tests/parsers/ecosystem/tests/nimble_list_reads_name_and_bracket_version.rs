use super::*;

#[test]
fn nimble_list_reads_name_and_bracket_version() {
    let out = "  jester  [0.5.0]\n  nimx  [0.1.0, 0.2.0]\n";
    let pkgs = nimble_list(out, "nimble").expect("this fixture parses");
    assert_eq!(pkgs[0].name, "jester");
    assert_eq!(pkgs[0].version.as_deref(), Some("0.5.0"));
    assert_eq!(pkgs[1].name, "nimx");
    assert_eq!(pkgs[1].version.as_deref(), Some("0.1.0"));
}
