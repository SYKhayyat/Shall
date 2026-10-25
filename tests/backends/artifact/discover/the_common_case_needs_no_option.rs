use super::*;

#[test]
fn the_common_case_needs_no_option() {
    let e = entries(&[
        ("fd-v10.2.0-x86_64-linux/fd", true),
        ("fd-v10.2.0-x86_64-linux/README.md", false),
        ("fd-v10.2.0-x86_64-linux/fd.1", false),
    ]);
    let found = find_executable(&e, "sharkdp/fd", None).unwrap();
    assert_eq!(normalise(&found), "fd-v10.2.0-x86_64-linux/fd");
}
