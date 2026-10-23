use super::*;

#[test]
fn winget_reports_the_id_and_the_available_version() {
    let p = parse_winget_outdated(WINGET);
    assert_eq!(p.len(), 2);
    assert_eq!(p[0].name, "7zip.7zip");
    assert_eq!(p[0].version.as_deref(), Some("26.02"));
    assert_eq!(p[1].name, "Adobe.Acrobat.Pro");
    assert_eq!(p[1].version.as_deref(), Some("26.001.21771"));
}
