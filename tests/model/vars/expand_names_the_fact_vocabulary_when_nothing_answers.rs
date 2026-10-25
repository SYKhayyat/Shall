use super::*;

#[test]
fn expand_names_the_fact_vocabulary_when_nothing_answers() {
    let facts = expand_facts();
    let err = expand("$rle", &facts, &origin(1)).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("home"),
        "the hint must list the facts: {}",
        msg
    );
}
