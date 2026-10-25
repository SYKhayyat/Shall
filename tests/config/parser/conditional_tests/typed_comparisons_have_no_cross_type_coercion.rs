use super::*;

#[test]
fn typed_comparisons_have_no_cross_type_coercion() {
    let mut vars = shall::model::vars::Vars::new();
    vars.insert("gpu".to_string(), Value::Bool(true));
    vars.insert("cores".to_string(), Value::Num(8.0));
    vars.insert("role".to_string(), Value::Str("travel".into()));
    vars.insert(
        "tags".into(),
        Value::List(vec![Value::Str("travel".into())]),
    );
    let f = facts().with_vars(vars);

    assert!(eval_when("$gpu == true", &f).unwrap());
    assert!(!eval_when("$gpu == false", &f).unwrap());
    // A boolean is not the string "true": no coercion (W2).
    assert!(!eval_when("$gpu == \"true\"", &f).unwrap());

    assert!(eval_when("$cores > 4", &f).unwrap());
    assert!(eval_when("$cores <= 8", &f).unwrap());
    assert!(!eval_when("$cores < 8", &f).unwrap());

    // `in` tests whether the left value is a member of the right list.
    assert!(eval_when("$role in [travel, work]", &f).unwrap());
    assert!(!eval_when("$role in [home, office]", &f).unwrap());
    // The right side may be a list-typed variable.
    assert!(eval_when("$role in $tags", &f).unwrap());
    // A non-list on the right of `in` is refused.
    assert!(eval_when("$role in $gpu", &f).is_err());
}
