use super::*;

#[test]
fn scoop_reports_the_latest_not_the_installed_version() {
    let p = parse_scoop_outdated(SCOOP);
    assert_eq!(p.len(), 2);
    assert_eq!(p[0].name, "7zip");
    assert_eq!(
        p[0].version.as_deref(),
        Some("26.02"),
        "the Latest column, not Installed — they sit next to each other and this table \
         has two columns whose header both end in `Version`"
    );
    assert_eq!(p[1].version.as_deref(), Some("1.36.3"));
}
