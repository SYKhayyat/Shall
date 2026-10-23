use super::*;

#[test]
fn composer_reads_latest_past_its_own_banner() {
    let p = parse_composer_outdated(REAL);
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].name, "psr/log");
    assert_eq!(
        p[0].version.as_deref(),
        Some("3.0.2"),
        "`version` is what is installed; `latest` is the answer"
    );
    assert_eq!(p[0].backend, "composer");
}
