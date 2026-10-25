use super::*;

#[test]
fn a_rule_round_trips_through_display() {
    for text in ["22/tcp", "53/udp", "default/incoming", "default/outgoing"] {
        assert_eq!(Rule::parse(text).unwrap().to_string(), text);
    }
}
