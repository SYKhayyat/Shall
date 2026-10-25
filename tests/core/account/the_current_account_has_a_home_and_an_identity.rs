use super::*;

#[test]
fn the_current_account_has_a_home_and_an_identity() {
    let account = Account::current().unwrap();
    assert!(!account.name().is_empty());
    assert!(!account.home().as_os_str().is_empty());
}
