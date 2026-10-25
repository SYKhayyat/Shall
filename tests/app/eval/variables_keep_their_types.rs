use super::*;

/// A variable keeps its type. Flattening to strings would hand a consumer `"3"` and
/// `"true"` and leave it guessing, which is the thing `Value` exists to prevent.
#[test]
fn variables_keep_their_types() {
    use shall::model::vars::Value;
    let mut state = shall::model::DesiredState::default();
    state.vars.insert("gpu".to_string(), Value::Bool(true));
    state.vars.insert("cores".to_string(), Value::Num(8.0));
    state
        .vars
        .insert("host".to_string(), Value::Str("aria".into()));
    let json = Evaluation::of(&state, Path::new("/repo")).render().unwrap();
    assert!(json.contains("\"gpu\": true"), "{}", json);
    assert!(json.contains("\"cores\": 8.0"), "{}", json);
    assert!(json.contains("\"host\": \"aria\""), "{}", json);
}
