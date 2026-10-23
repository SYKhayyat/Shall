use super::*;

#[test]
fn ws_name_version_parses_and_skips_header() {
    let out = "NAME       VERSION\nfoo        1.2.3\nbar        0.1.0   some-desc\n";
    let pkgs = ws_name_version(out, "helm").expect("this fixture parses");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "foo");
    assert_eq!(pkgs[0].version.as_deref(), Some("1.2.3"));
    assert_eq!(pkgs[1].name, "bar");
    assert_eq!(pkgs[0].backend, "helm");
}
