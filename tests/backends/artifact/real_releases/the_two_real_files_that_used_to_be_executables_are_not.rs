//! The two real files that made the rule stricter, named so the reason outlives the fixture:
//! `MD5SUMS` is in every rclone release and `jq-linux64` is in jq's.

use super::*;

#[test]
fn the_two_real_files_that_used_to_be_executables_are_not() {
    for (os, arch) in [
        ("linux", "x86_64"),
        ("windows", "x86_64"),
        ("macos", "aarch64"),
    ] {
        let p = Platform::new(os, arch);
        assert_ne!(
            classify_format("MD5SUMS", &p, None),
            Some(Format::Binary),
            "MD5SUMS on {}/{}",
            os,
            arch
        );
    }
    // `linux64` is a real spelling of "linux", so on Windows this is foreign now rather than
    // a nameless file that could run anywhere.
    assert!(!Platform::new("windows", "x86_64").accepts("jq-linux64"));
    assert!(Platform::new("linux", "x86_64").accepts("jq-linux64"));
    // ...and on Linux it is an executable, because there it does name the machine.
    assert_eq!(
        classify_format("jq-linux64", &Platform::new("linux", "x86_64"), None),
        Some(Format::Binary)
    );
}
