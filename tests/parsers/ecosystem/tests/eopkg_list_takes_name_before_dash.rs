use super::*;

#[test]
fn eopkg_list_takes_name_before_dash() {
    let out = "nano - Small, friendly text editor\ngit - Distributed VCS\n";
    let pkgs = eopkg_list(out, "eopkg").expect("this fixture parses");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "nano");
    assert_eq!(pkgs[1].name, "git");
}
