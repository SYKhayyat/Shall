use super::*;

#[test]
fn a_depends_row_is_matched_by_its_key_and_not_by_its_column() {
    assert_eq!(parse_depends_on(SI_JQ), vec!["glibc", "oniguruma"]);
}
