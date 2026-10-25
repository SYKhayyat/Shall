use super::*;

#[test]
fn a_store_that_cannot_address_a_named_account_refuses_before_writing() {
    let core = SettingBackendCore::new(CommandExecutor::new(true, false), adapters(vec![]));
    let account = shall::core::account::Account::current().unwrap();
    let target = StoreTarget {
        account: Some(account.clone()),
    };
    let addressable = target.is_addressable(&registry());
    if addressable {
        let hive = target.hive();
        assert!(hive.starts_with("HKEY_USERS"), "{hive}");
        assert!(hive.contains(&account.sid().expect("a SID on this platform")));
        assert!(core
            .target_of(&registry(), Some(account.name()), "X/k")
            .is_ok());
    } else {
        let err = core
            .target_of(&registry(), Some(account.name()), "X/k")
            .expect_err("a store with no address for a named account must refuse")
            .to_string();
        assert!(err.contains(account.name()), "{err}");
        assert!(err.contains("nothing was written"), "{err}");
        assert!(err.contains("windows-registry"), "{err}");
    }
}
