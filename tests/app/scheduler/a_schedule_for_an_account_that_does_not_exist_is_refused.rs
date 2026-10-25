use super::*;

#[test]
fn a_schedule_for_an_account_that_does_not_exist_is_refused() {
    let err = TaskTarget::resolve(Some("shall-no-such-account-71"))
        .expect_err("an unresolvable account must be refused")
        .to_string();
    assert!(err.contains("shall-no-such-account-71"), "{err}");
}
