use super::*;

#[test]
fn a_commit_with_an_empty_subject_is_not_a_parse_failure() {
    let line = "a1111111111111111111111111111111111111111\t2026-03-14\t\n";
    assert_eq!(introduced_in(line).unwrap().subject, "");
}
