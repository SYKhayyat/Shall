use super::*;

#[test]
fn unknown_accounts_are_refused() {
    assert!(Account::resolve("shall-no-such-account-71").is_err());
}
