use super::*;

#[test]
fn a_derivative_resolves_to_the_family_that_decides_the_artifact() {
    let mint = "NAME=\"Linux Mint\"\nID=linuxmint\nID_LIKE=\"ubuntu debian\"\n";
    assert_eq!(parse_os_release_family(mint).as_deref(), Some("debian"));
}
