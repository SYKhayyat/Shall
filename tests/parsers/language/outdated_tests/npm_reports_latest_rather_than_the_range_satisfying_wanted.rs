use super::*;

/// `wanted` is the newest release inside a package.json range. A global install has no
/// package.json, so honouring a caret nobody wrote would hide a major version.
#[test]
fn npm_reports_latest_rather_than_the_range_satisfying_wanted() {
    let out = r#"{"typescript":{"current":"5.4.0","wanted":"5.6.0","latest":"5.7.2"}}"#;
    let p = parse_npm_outdated(out, "npm");
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].name, "typescript");
    assert_eq!(p[0].version.as_deref(), Some("5.7.2"));
    assert_eq!(p[0].backend, "npm");
}
