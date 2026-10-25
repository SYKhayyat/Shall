use super::*;

#[test]
fn a_repeat_is_ignored() {
    assert_eq!(on("Work\nWork\n", "any"), ["Work"]);
}
