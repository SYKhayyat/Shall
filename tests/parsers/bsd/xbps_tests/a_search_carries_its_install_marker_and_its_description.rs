use super::*;

#[test]
fn a_search_carries_its_install_marker_and_its_description() {
    let out = "[*] bash-5.2.15_2   The GNU Bourne Again Shell\n[-] zsh-5.9_2   The Z shell\n";
    let pkgs = parse_xbps_search(out);
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "bash");
    assert_eq!(
        pkgs[0].properties.get("description").map(String::as_str),
        Some("The GNU Bourne Again Shell")
    );
    assert_eq!(pkgs[1].name, "zsh");
    assert_eq!(pkgs[1].version.as_deref(), Some("5.9_2"));
}
