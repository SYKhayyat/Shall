use super::*;

#[test]
fn comments_and_blanks_are_skipped() {
    let p = parse("# which managers\n\napt   # the system one\ncargo\n").unwrap();
    assert_eq!(p.order(), ["apt", "cargo"]);
}
