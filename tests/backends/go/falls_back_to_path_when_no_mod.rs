use super::*;

#[test]
fn falls_back_to_path_when_no_mod() {
    let out = "/root/go/bin/tool: go1.21.0\n\tpath\texample.com/tool\n";
    let (module, version) = parse_go_version_m(out).unwrap();
    assert_eq!(module, "example.com/tool");
    assert_eq!(version, None);
}
