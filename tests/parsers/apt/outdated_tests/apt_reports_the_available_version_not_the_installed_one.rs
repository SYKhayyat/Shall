use super::*;

#[test]
fn apt_reports_the_available_version_not_the_installed_one() {
    let p = parse_apt_outdated(APT);
    assert_eq!(p.len(), 3);
    assert_eq!(p[0].name, "gzip");
    assert_eq!(
        p[0].version.as_deref(),
        Some("1.12-1ubuntu3.2"),
        "the bracketed version is what is INSTALLED; reading it reports everything as \
         already up to date"
    );
    assert_eq!(p[2].name, "libpam-runtime");
    assert_eq!(p[2].version.as_deref(), Some("1.5.3-5ubuntu5.6"));
}
