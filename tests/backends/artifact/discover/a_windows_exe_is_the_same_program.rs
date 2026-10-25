use super::*;

#[test]
fn a_windows_exe_is_the_same_program() {
    let e = entries(&[("fd.exe", true), ("LICENSE", false)]);
    let found = find_executable(&e, "sharkdp/fd", None).unwrap();
    assert_eq!(normalise(&found), "fd.exe");
}
