use super::*;

#[test]
fn a_chain_can_end_in_the_whole_priority_list() {
    assert_eq!(
        cands("apt,list:ripgrep"),
        (None, Candidates::NamedThenPriority(vec!["apt".into()]))
    );
}
