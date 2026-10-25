use super::*;

#[test]
fn a_message_about_a_variable_block_says_what_the_variable_is() {
    // The point of W8's messaging: `active` says the condition, and the file the reader
    // would open next does not contain the value.
    let facts = facts_with(&[("role", "desktop")]);
    assert_eq!(
        describe_gate("$role == travel", &facts),
        "`when $role == travel` ($role is desktop)"
    );
    // No variable, nothing to add: the condition is already the whole answer.
    assert_eq!(
        describe_gate("host == laptop", &facts),
        "`when host == laptop`"
    );
}
