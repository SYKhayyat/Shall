use super::*;

#[test]
fn expand_stringifies_a_number_or_boolean() {
    let mut facts = expand_facts();
    facts.vars.insert("n".to_string(), Value::Num(5.0));
    assert_eq!(expand("v$n", &facts, &origin(1)).unwrap(), "v5");
}
