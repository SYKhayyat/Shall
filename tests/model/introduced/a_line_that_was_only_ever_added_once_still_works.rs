use super::*;

#[test]
fn a_line_that_was_only_ever_added_once_still_works() {
    let one = "a1111111111111111111111111111111111111111\t2026-03-14\tadd ripgrep\n";
    assert_eq!(introduced_in(one).unwrap().subject, "add ripgrep");
}
