use super::*;

#[test]
fn a_plan_with_no_vars_field_deserializes_to_empty() {
    // The field is `serde(default)`, so a plan written before it existed still reads.
    let raw = r#"{"schema":2,"created_at":null,"desired_hash":"x","installs":[],"removals":[]}"#;
    let plan: SavedPlan = serde_json::from_str(raw).unwrap();
    assert!(plan.vars.is_empty());
}
