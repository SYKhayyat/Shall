use super::*;

#[test]
fn a_dash_in_the_name_is_not_mistaken_for_a_version() {
    // Alpine names really do look like this. Split blind, `xz-libs-dev` became name
    // `xz` — and this is the installed lister, so `info()` could never find the real
    // package and `remove` silently did nothing.
    let res = parse_dash_version_list("xz-libs-dev\npy3-requests-2.31.0-r0\nbash-5.2.15\n", "apk")
        .expect("this fixture parses");
    assert_eq!(res[0].name, "xz-libs-dev");
    assert_eq!(res[0].version, None);
    assert_eq!(res[1].name, "py3-requests");
    assert_eq!(res[1].version.as_deref(), Some("2.31.0-r0"));
    // No `-rN` revision: the version is still a version.
    assert_eq!(res[2].name, "bash");
    assert_eq!(res[2].version.as_deref(), Some("5.2.15"));
}
