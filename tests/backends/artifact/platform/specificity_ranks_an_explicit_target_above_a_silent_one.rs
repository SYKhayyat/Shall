use super::*;

#[test]
fn specificity_ranks_an_explicit_target_above_a_silent_one() {
    assert_eq!(linux64().specificity("fd-linux-x86_64.tar.gz"), 2);
    assert_eq!(linux64().specificity("fd-linux.tar.gz"), 1);
    assert_eq!(linux64().specificity("fd.tar.gz"), 0);
}
