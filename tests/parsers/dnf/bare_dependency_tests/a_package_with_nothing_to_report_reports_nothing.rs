use super::*;

#[test]
fn a_package_with_nothing_to_report_reports_nothing() {
    assert!(parse_bare_dependency_names("").is_empty());
}
