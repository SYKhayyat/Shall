use super::*;

#[test]
fn expand_refuses_a_list_rather_than_joining_it() {
    let mut facts = expand_facts();
    facts
        .vars
        .insert("tags".to_string(), Value::List(vec![str_val("a")]));
    let err = expand("x-$tags", &facts, &origin(1)).unwrap_err();
    assert!(err.what.contains("list"), "{}", err);
}
