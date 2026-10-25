use super::*;

#[test]
fn a_counting_dated_line_beats_an_undated_one() {
    // II.7 rule 6 — the ONLY exception to rule 5.
    let dated = decl("modules/temp.txt", 1, &[("expires", "2026-07-20T00:00")]);
    let plain = decl("modules/base.txt", 2, &[("version", "1.6")]);
    let out = reconcile("apt:jq", dated, plain, now()).unwrap();
    assert_eq!(out.origin.file.to_string_lossy(), "modules/temp.txt");
}
