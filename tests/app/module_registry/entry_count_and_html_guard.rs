use super::*;

#[test]
fn entry_count_and_html_guard() {
    let body = "# header\n\napt:htop\ncargo:ripgrep\n# note\n";
    assert_eq!(count_entries(body), 2);
    assert!(looks_like_html("<!DOCTYPE html>"));
    assert!(!looks_like_html("apt:htop"));
}
