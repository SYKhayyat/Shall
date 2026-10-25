use super::*;

#[tokio::test]
async fn a_named_accounts_shim_lands_in_that_accounts_own_bin_directory() {
    let tmp = tempdir().unwrap();
    let configured = tmp.path().join("configured-bin");
    let account = Account::current().unwrap();
    let expected = account.bin_dir().unwrap();

    let (dir, owner) = shim_target(&configured, Some(account.name())).unwrap();
    assert_eq!(dir, expected, "the target is not the account's own bin dir");
    assert!(owner.is_some(), "no account came back to own the file");
    assert_ne!(
        dir, configured,
        "a named account resolved to Shall's configured bin dir"
    );

    let mgr = ShimManager::for_user(&configured, Some(account.name()))
        .await
        .unwrap();
    assert_eq!(mgr.bin_dir(), expected);

    let (dir, owner) = shim_target(&configured, None).unwrap();
    assert_eq!(dir, configured);
    assert!(owner.is_none());
}
