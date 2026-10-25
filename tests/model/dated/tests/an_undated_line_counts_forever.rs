use super::*;

#[test]
fn an_undated_line_counts_forever() {
    assert_eq!(dating_of(&opts(&[]), at("2030-01-01")), Dating::Undated);
    assert!(dating_of(&opts(&[]), at("2030-01-01")).counts());
}
