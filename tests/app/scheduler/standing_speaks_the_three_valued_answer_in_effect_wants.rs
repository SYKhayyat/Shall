use super::*;

#[test]
fn standing_speaks_the_three_valued_answer_in_effect_wants() {
    assert_eq!(Standing::Matches.in_effect(), Some(true));
    assert_eq!(Standing::Missing.in_effect(), Some(false));
    assert_eq!(Standing::Differs("x").in_effect(), Some(false));
    assert_eq!(Standing::Unknown.in_effect(), None);
}
