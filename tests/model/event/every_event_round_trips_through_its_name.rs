use super::*;

#[test]
fn every_event_round_trips_through_its_name() {
    for event in Event::ALL {
        assert_eq!(Event::parse(event.as_str()), Some(event));
    }
}
