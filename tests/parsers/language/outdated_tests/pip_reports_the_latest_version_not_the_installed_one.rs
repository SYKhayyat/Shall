use super::*;

#[test]
fn pip_reports_the_latest_version_not_the_installed_one() {
    let p = parse_pip_outdated(PIP);
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].name, "pip");
    assert_eq!(
        p[0].version.as_deref(),
        Some("26.2.1"),
        "`version` is what is installed; reading it as the available one makes every \
         package look current"
    );
}
