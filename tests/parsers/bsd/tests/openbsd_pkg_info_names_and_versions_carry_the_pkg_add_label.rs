use super::*;

#[test]
fn openbsd_pkg_info_names_and_versions_carry_the_pkg_add_label() {
    let input = "wget-1.21.4        retrieve files over HTTP/FTP\n\
                 gmake-4.4          GNU make\n";
    let res = parse_pkg_add(input).expect("real `pkg_info` output");
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].name, "wget");
    assert_eq!(res[0].version.as_deref(), Some("1.21.4"));
    assert_eq!(
        res[0].backend, "pkg_add",
        "an OpenBSD package is a pkg_add package"
    );
}
