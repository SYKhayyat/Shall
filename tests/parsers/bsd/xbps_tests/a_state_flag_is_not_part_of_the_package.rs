use super::*;

#[test]
fn a_state_flag_is_not_part_of_the_package() {
    let out = "\
ii bash-5.2.15_2            The GNU Bourne Again Shell
ii xbps-triggers-0.128_1    XBPS triggers for Void Linux
";
    let pkgs = parse_xbps_list(out).expect("real xbps -l output");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "bash");
    assert_eq!(pkgs[0].version.as_deref(), Some("5.2.15_2"));
    assert_eq!(pkgs[0].backend, "xbps");
    // A name with an internal dash must not be split at the wrong hyphen.
    assert_eq!(pkgs[1].name, "xbps-triggers");
    assert_eq!(pkgs[1].version.as_deref(), Some("0.128_1"));
}
