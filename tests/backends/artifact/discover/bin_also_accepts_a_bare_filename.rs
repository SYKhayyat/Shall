use super::*;

#[test]
fn bin_also_accepts_a_bare_filename() {
    let e = entries(&[("deep/nested/tool", true)]);
    let found = find_executable(&e, "x/y", Some("tool")).unwrap();
    assert_eq!(normalise(&found), "deep/nested/tool");
}
