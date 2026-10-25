use super::*;

#[test]
fn the_payload_is_versioned_and_names_its_event() {
    let p = Payload::new(Event::OnDrift, json!({"removed": ["jq"]}));
    let out = p.to_stdin();
    assert!(out.contains("\"schema\":1"), "{}", out);
    assert!(out.contains("\"event\":\"on_drift\""), "{}", out);
    assert!(out.contains("\"removed\":[\"jq\"]"), "{}", out);
}
