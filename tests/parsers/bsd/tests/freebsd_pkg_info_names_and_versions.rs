use super::*;

#[test]
fn freebsd_pkg_info_names_and_versions() {
    let input = "pkg-1.20.9                     Package manager\n\
                 py311-requests-2.31.0          HTTP library\n\
                 vim-9.0.1897                   improved vi\n";
    let res = parse_pkg(input).expect("real `pkg info` output");
    assert_eq!(res.len(), 3);
    assert_eq!(res[0].name, "pkg");
    assert_eq!(res[0].version.as_deref(), Some("1.20.9"));
    assert_eq!(res[0].backend, "pkg");
    assert_eq!(res[1].name, "py311-requests");
    assert_eq!(res[1].version.as_deref(), Some("2.31.0"));
}
