use super::*;

#[test]
fn the_name_match_outranks_the_lone_executable_rule() {
    let e = entries(&[("fd", false), ("helper", true)]);
    let found = find_executable(&e, "sharkdp/fd", None).unwrap();
    assert_eq!(normalise(&found), "fd");
}
