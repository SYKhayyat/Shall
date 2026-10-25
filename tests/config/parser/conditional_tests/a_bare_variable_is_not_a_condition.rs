use super::*;

#[test]
fn a_bare_variable_is_not_a_condition() {
    // W3: `false` is a non-empty string, so a truthiness test would fire on `gpu = false`.
    let mut vars = shall::model::vars::Vars::new();
    vars.insert("gpu".to_string(), Value::Bool(false));
    let f = facts().with_vars(vars);
    let err = eval_when("$gpu", &f).unwrap_err();
    assert!(err.to_string().contains("== true"), "{}", err);
}
