use super::*;

#[test]
fn expand_prefers_a_variable_you_decided_over_a_fact_the_machine_reported() {
    // IX.4 in values: a `home = …` in `vars` wins, so adding a fact never moves a file
    // that named a variable the same thing.
    let mut facts = expand_facts();
    facts.vars.insert("home".to_string(), str_val("/elsewhere"));
    assert_eq!(
        expand("${home}/Documents", &facts, &origin(1)).unwrap(),
        "/elsewhere/Documents"
    );
}
