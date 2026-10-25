use super::*;

#[test]
fn a_lapsed_line_stops_counting_and_the_undated_one_wins() {
    // The dated line lingers in the file; it just stops having an opinion.
    let lapsed = decl("modules/temp.txt", 1, &[("expires", "2026-01-01T00:00")]);
    let plain = decl("modules/base.txt", 2, &[("version", "1.6")]);
    let out = reconcile("apt:jq", lapsed, plain, now()).unwrap();
    assert_eq!(out.origin.file.to_string_lossy(), "modules/base.txt");
    assert_eq!(out.options.one("version"), Some("1.6"));
}
