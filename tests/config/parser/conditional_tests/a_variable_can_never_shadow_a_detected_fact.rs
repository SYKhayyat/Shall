use super::*;

#[test]
fn a_variable_can_never_shadow_a_detected_fact() {
    // IX.4: the sigil exists so Shall can add a detected fact forever without changing the
    // meaning of a file where someone used that word as a variable name.
    let mut vars = shall::model::vars::Vars::new();
    vars.insert("os".to_string(), Value::Str("definitely-not-linux".into()));
    let f = facts().with_vars(vars);
    assert!(
        eval_when("os == linux", &f).unwrap(),
        "`os` must stay the detected fact"
    );
    assert!(eval_when("$os == definitely-not-linux", &f).unwrap());
}
