use super::*;

#[test]
fn ordering_a_string_is_refused_not_answered_wrongly() {
    let mut vars = shall::model::vars::Vars::new();
    vars.insert("ver".to_string(), Value::Str("10".into()));
    let f = facts().with_vars(vars);
    // `"10" > "9"` is false under string ordering and true under intuition; refuse it (W2).
    assert!(eval_when("$ver > 9", &f).is_err());
}
