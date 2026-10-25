use super::*;

#[test]
fn an_unknown_distribution_reports_its_own_name_rather_than_nothing() {
    assert_eq!(
        parse_os_release_family("ID=voidlinux\n").as_deref(),
        Some("voidlinux")
    );
}
