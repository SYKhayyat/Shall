use super::*;

#[test]
fn one_name_per_line_and_blank_lines_are_not_packages() {
    assert_eq!(
        parse_bare_dependency_names("glibc\n\noniguruma\n  \n"),
        vec!["glibc", "oniguruma"]
    );
}
