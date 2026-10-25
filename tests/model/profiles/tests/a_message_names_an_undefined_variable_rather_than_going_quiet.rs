use super::*;

#[test]
fn a_message_names_an_undefined_variable_rather_than_going_quiet() {
    assert_eq!(
        describe_gate("$role == travel", &facts()),
        "`when $role == travel` ($role is undefined)"
    );
}
