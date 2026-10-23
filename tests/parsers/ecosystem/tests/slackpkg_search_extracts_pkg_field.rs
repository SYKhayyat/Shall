use super::*;

#[test]
fn slackpkg_search_extracts_pkg_field() {
    let out = "[ installed ] - mc-4.8.29-x86_64-1\n[uninstalled] - htop-3.2.1-x86_64-1\n";
    let pkgs = slackpkg_search(out, "slackpkg");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "mc");
    assert_eq!(pkgs[1].name, "htop");
}
