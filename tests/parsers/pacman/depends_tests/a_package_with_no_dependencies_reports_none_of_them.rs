use super::*;

#[test]
fn a_package_with_no_dependencies_reports_none_of_them() {
    assert!(parse_depends_on("Depends On      : None\n").is_empty());
    assert!(parse_depends_on("Name            : tree\n").is_empty());
}
