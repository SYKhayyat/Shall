use super::*;

/// Both readers of the same output, so the pair cannot drift apart again.
#[test]
fn the_outdated_probe_reads_the_same_output_and_reports_the_latest() {
    let outdated = parse_composer_outdated(REAL_GLOBAL_SHOW);
    let names: Vec<&str> = outdated.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["psr/log", "symfony/console"]);
    assert_eq!(outdated[0].version.as_deref(), Some("3.0.2"));
}
