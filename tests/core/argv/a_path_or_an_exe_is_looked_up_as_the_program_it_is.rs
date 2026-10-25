use super::*;

#[test]
fn a_path_or_an_exe_is_looked_up_as_the_program_it_is() {
    assert!(terminates_options("/usr/bin/apt-get"));
    assert!(terminates_options(r"C:\tools\cargo.exe"));
    assert!(!terminates_options(r"C:\Users\me\scoop\shims\scoop.exe"));
}
