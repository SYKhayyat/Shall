use super::*;

#[test]
fn a_manager_that_reads_the_terminator_as_a_name_is_not_listed_as_terminating() {
    assert!(!terminates_options("asdf"));
    assert!(!terminates_options("spack"));
    // The controls: the ones measured in the same sweep that DO honour it stay listed, so
    // this is not a test that would pass if the table were simply emptied.
    assert!(terminates_options("cargo"));
    assert!(terminates_options("npm"));
    assert!(terminates_options("pipx"));
    assert!(terminates_options("luarocks"));
}
