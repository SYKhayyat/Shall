use super::*;

#[test]
fn comments_and_blanks_are_skipped() {
    assert_eq!(on("# what I am\n\nWork   # for work\n", "any"), ["Work"]);
}
