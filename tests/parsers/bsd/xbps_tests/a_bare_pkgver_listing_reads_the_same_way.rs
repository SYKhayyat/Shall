use super::*;

#[test]
fn a_bare_pkgver_listing_reads_the_same_way() {
    let pkgs = parse_xbps_list("curl-8.4.0_1\ngit-2.42.0_1\n").expect("bare pkgver listing");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "curl");
    assert_eq!(pkgs[1].name, "git");
}
