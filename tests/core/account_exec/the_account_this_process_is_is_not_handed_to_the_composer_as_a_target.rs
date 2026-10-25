use super::*;

#[cfg(unix)]
#[test]
fn the_account_this_process_is_is_not_handed_to_the_composer_as_a_target() {
    let account = Account::current().expect("this process has an account");
    assert!(account.is_this_process());
    assert!(Some(&account)
        .filter(|candidate| !candidate.is_this_process())
        .is_none());
}
