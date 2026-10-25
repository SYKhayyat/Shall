use super::*;

#[test]
fn an_exact_name_wins_over_a_sibling_that_merely_starts_with_it() {
    let e = entries(&[("bin/fd", true), ("bin/fdfind", true)]);
    let found = find_executable(&e, "sharkdp/fd", None).unwrap();
    assert_eq!(normalise(&found), "bin/fd");
}
