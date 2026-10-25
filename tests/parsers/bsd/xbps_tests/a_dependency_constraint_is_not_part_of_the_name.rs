use super::*;

#[test]
fn a_dependency_constraint_is_not_part_of_the_name() {
    assert_eq!(
        parse_xbps_dependencies("glibc>=2.36_1\noniguruma>=6.9\n\n"),
        vec!["glibc", "oniguruma"]
    );
}
