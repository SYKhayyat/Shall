use super::*;

#[test]
fn npm_registry_empty_objects() {
    let json: serde_json::Value = serde_json::from_str(r#"{"objects":[]}"#).unwrap();
    assert!(parse_npm_registry(&json, "pnpm").is_empty());
}
