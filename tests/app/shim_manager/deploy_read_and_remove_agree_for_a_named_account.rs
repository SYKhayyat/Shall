use super::*;

#[tokio::test]
async fn deploy_read_and_remove_agree_for_a_named_account() {
    let tmp = tempdir().unwrap();
    let configured = tmp.path().join("configured-bin");
    let account = Account::current().unwrap();
    let here = ShimManager::for_user(&configured, None).await.unwrap();
    let there = ShimManager::for_user(&configured, Some(account.name()))
        .await
        .unwrap();

    there.create_shim("ripgrep").await.unwrap();
    let deployed = there.shim_path("ripgrep");
    assert!(
        deployed.exists(),
        "the shim was not deployed where the manager says it goes: {deployed:?}"
    );
    assert!(there.is_in_effect("ripgrep").await);
    assert!(
        !here.is_in_effect("ripgrep").await,
        "a shim deployed for an account is not in effect in Shall's own bin dir, and \
         reporting it there would place a second copy"
    );

    there
        .remove_shim(
            "ripgrep",
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
        )
        .await
        .unwrap();
    assert!(!deployed.exists(), "removal looked somewhere else");
    assert!(!there.is_in_effect("ripgrep").await);
}
