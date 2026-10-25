use super::*;

/// One envelope for every event, so a hook handling two parses one shape.
#[test]
fn every_event_produces_the_same_envelope() {
    for event in Event::ALL {
        let v: serde_json::Value =
            serde_json::from_str(&Payload::new(event, json!({})).to_stdin()).unwrap();
        assert_eq!(v["schema"], 1);
        assert_eq!(v["event"], event.as_str());
        assert!(v.get("data").is_some());
    }
}
