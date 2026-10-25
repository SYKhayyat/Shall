use super::*;

#[test]
fn a_lapsed_line_stops_counting_but_is_reported_with_its_file_and_line() {
    // II.16: expired lines linger — Shall must not rewrite your files — so it mentions
    // them, naming the exact file and line, never vaguely.
    let f = fx(
        "Work\n",
        &[("Work", "use base\n")],
        &[("base.txt", "apt:curl\napt:jq@expires=2026-01-01\n")],
    );
    let d = resolve(&f).unwrap();
    assert_eq!(names(&d, "apt"), ["curl"], "the lapsed jq stops counting");
    assert_eq!(d.lapsed.len(), 1);
    assert_eq!(d.lapsed[0].0, "apt:jq");
    assert!(d.lapsed[0].1.to_string().contains("base.txt:2"));
}
