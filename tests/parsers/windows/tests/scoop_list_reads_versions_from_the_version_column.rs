use super::*;

#[test]
fn scoop_list_reads_versions_from_the_version_column() {
    let res = parse_scoop_list(&scoop_list_fixture()).expect("the fixture has a header row");
    let seven = res.iter().find(|p| p.name == "7zip").unwrap();
    assert_eq!(seven.version.as_deref(), Some("26.00"));
    // The date must never reach a version field — that is what the shifted read did.
    assert!(
        res.iter().all(|p| !p
            .version
            .as_deref()
            .unwrap_or_default()
            .starts_with("2026-")),
        "an Updated timestamp was parsed as a version: {:?}",
        res.iter()
            .map(|p| (&p.name, &p.version))
            .collect::<Vec<_>>()
    );
}
