use super::*;

#[test]
fn parses_module_and_version_from_mod_line() {
    let out = "/root/go/bin/fzf: go1.21.0\n\tpath\tgithub.com/junegunn/fzf\n\tmod\tgithub.com/junegunn/fzf\tv0.42.0\th1:abcd\n";
    let (module, version) = parse_go_version_m(out).unwrap();
    assert_eq!(module, "github.com/junegunn/fzf");
    assert_eq!(version.as_deref(), Some("v0.42.0"));
}
