use super::*;

#[cfg(unix)]
#[tokio::test]
async fn a_shim_deployed_for_an_account_is_owned_by_that_account() {
    let tmp = tempdir().unwrap();
    let configured = tmp.path().join("configured-bin");
    let account = Account::current().unwrap();
    let mgr = ShimManager::for_user(&configured, Some(account.name()))
        .await
        .unwrap();
    mgr.create_shim("ripgrep").await.unwrap();
    let deployed = mgr.shim_path("ripgrep");
    assert!(
        account.owns(&deployed, true).unwrap(),
        "the deployed shim is owned by somebody other than the account it is for"
    );
}
