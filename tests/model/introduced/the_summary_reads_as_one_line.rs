use super::*;

#[test]
fn the_summary_reads_as_one_line() {
    let s = introduced_in(LOG).unwrap().summary();
    assert!(s.starts_with("a111111"), "{}", s);
    assert!(s.contains("2026-03-14"), "{}", s);
    assert!(s.contains("add ripgrep"), "{}", s);
}
