use super::*;

#[test]
fn a_plan_carries_its_variables_across_a_round_trip() {
    use shall::model::vars::Value;
    let mut plan = SavedPlan::from_changes(
        &shall::app::sync::planner::SyncChanges::default(),
        &shall::app::apply::ResourceChanges::default(),
        Some(1),
    );
    plan.vars
        .insert("role".to_string(), Value::Str("travel".into()));
    plan.vars.insert("cores".to_string(), Value::Num(8.0));
    let json = serde_json::to_string(&plan).unwrap();
    let back: SavedPlan = serde_json::from_str(&json).unwrap();
    assert_eq!(back.vars["role"], Value::Str("travel".into()));
    assert_eq!(back.vars["cores"], Value::Num(8.0));
}
