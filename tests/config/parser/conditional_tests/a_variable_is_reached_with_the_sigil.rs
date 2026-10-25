use super::*;

#[test]
fn a_variable_is_reached_with_the_sigil() {
    let f = with_role("travel");
    assert!(eval_when("$role == travel", &f).unwrap());
    assert!(!eval_when("$role == desktop", &f).unwrap());
    assert!(eval_when("$role in [travel, workstation]", &f).unwrap());
}
