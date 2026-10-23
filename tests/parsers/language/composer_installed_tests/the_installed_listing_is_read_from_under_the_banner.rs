use super::*;

#[test]
fn the_installed_listing_is_read_from_under_the_banner() {
    let found = parse_installed("composer", REAL_GLOBAL_SHOW).expect("a readable listing");
    let names: Vec<&str> = found.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["psr/log", "symfony/console"]);
    // `version`, not `latest` — the second is what an upgrade would move to.
    assert_eq!(found[0].version.as_deref(), Some("1.1.4"));
}
