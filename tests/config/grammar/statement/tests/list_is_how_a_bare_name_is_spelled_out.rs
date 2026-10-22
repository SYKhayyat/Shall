use super::*;

#[test]
fn list_is_how_a_bare_name_is_spelled_out() {
    assert_eq!(cands("ripgrep"), (None, Candidates::Priority));
    assert_eq!(cands("list:ripgrep"), (None, Candidates::Priority));
}
