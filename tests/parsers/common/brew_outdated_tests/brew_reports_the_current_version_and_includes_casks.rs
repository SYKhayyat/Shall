use super::*;

#[test]
fn brew_reports_the_current_version_and_includes_casks() {
    let p = parse_brew_outdated(BREW);
    assert_eq!(p.len(), 3);
    assert_eq!(p[0].name, "jq");
    assert_eq!(p[0].version.as_deref(), Some("1.7.1"));
    assert_eq!(
        p[2].name, "firefox",
        "a cask is installed by the same declaration, so its updates are updates"
    );
    assert_eq!(p[2].version.as_deref(), Some("141.0"));
}
