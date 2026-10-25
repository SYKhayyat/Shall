use super::*;

#[test]
fn expand_of_an_undetectable_fact_is_an_error_not_an_empty_string() {
    let mut facts = expand_facts();
    facts.home = None;
    let err = expand("${home}/Documents", &facts, &origin(1)).unwrap_err();
    assert!(err.what.contains("home"), "{}", err);
}
