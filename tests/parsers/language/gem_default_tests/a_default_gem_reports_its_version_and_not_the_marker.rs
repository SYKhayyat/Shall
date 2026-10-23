use super::*;

/// `gem list --local` marks the gems that ship with Ruby, and the marker is not part of
/// the version. Verbatim from this host.
#[test]
fn a_default_gem_reports_its_version_and_not_the_marker() {
    let out = "bigdecimal (4.0.1)\nbundler (default: 4.0.10)\ncsv (3.3.5, 3.3.4)\n";
    let p = parse_installed("gem", out).expect("this fixture parses");
    assert_eq!(p.len(), 3);
    assert_eq!(p[0].version.as_deref(), Some("4.0.1"));
    assert_eq!(
        p[1].version.as_deref(),
        Some("4.0.10"),
        "`default:` is RubyGems saying the gem ships with Ruby, not part of the version"
    );
    // The existing behaviour for a multi-version gem is unchanged: the newest wins.
    assert_eq!(p[2].version.as_deref(), Some("3.3.5"));
}
