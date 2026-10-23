use super::*;

/// Verbatim from `gem outdated` on this host.
#[test]
fn gem_reads_the_right_side_of_the_comparison() {
    let out = "bigdecimal (4.0.1 < 4.1.2)\nbundler (4.0.10 < 4.0.18)\ncsv (3.3.5 < 3.3.6)\n";
    let p = parse_gem_outdated(out);
    assert_eq!(p.len(), 3);
    assert_eq!(p[0].name, "bigdecimal");
    assert_eq!(p[0].version.as_deref(), Some("4.1.2"));
    assert_eq!(p[2].version.as_deref(), Some("3.3.6"));
}
