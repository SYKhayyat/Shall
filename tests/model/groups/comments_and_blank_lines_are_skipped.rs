use super::*;

#[test]
fn comments_and_blank_lines_are_skipped() {
    let g = Groups::parse("# my groups\n\ntools = apt, cargo  # the ones I use\n").unwrap();
    assert_eq!(g.expand("tools").unwrap().len(), 2);
}
