use super::*;

#[test]
fn a_default_policy_parses_in_both_directions() {
    assert_eq!(
        Rule::parse("default/incoming"),
        Ok(Rule::Default {
            direction: Direction::Incoming
        })
    );
    assert_eq!(
        Rule::parse("default/outgoing"),
        Ok(Rule::Default {
            direction: Direction::Outgoing
        })
    );
}
