use super::*;

#[test]
fn bin_names_the_executable_inside_an_archive() {
    assert_eq!(
        options_of("github:foo/bar@bin=build/bar").one("bin"),
        Some("build/bar")
    );
}
