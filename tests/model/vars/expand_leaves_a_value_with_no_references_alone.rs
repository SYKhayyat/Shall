use super::*;

#[test]
fn expand_leaves_a_value_with_no_references_alone() {
    let facts = expand_facts();
    assert_eq!(
        expand("plain/path", &facts, &origin(1)).unwrap(),
        "plain/path"
    );
}
