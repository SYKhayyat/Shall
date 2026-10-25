use super::*;

#[test]
fn bin_names_the_executable_by_path() {
    let e = entries(&[("bin/fd", true), ("bin/fdfind", true)]);
    let found = find_executable(&e, "sharkdp/fd", Some("bin/fdfind")).unwrap();
    assert_eq!(normalise(&found), "bin/fdfind");
}
