use super::*;

/// A removal Shall cannot address must refuse, not run a command that matches nothing.
#[tokio::test]
async fn a_repo_named_by_something_that_is_not_a_url_is_refused() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let mgr = apk_repo(mock.clone(), vfs);
    let err = mgr
        .remove_repo(
            "testing",
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
        )
        .await
        .expect_err("apk knows no repository called `testing`")
        .to_string();
    assert!(err.contains("testing"), "{}", err);
    assert!(err.contains("repo list"), "{}", err);
    assert!(
        !mock
            .get_calls()
            .await
            .iter()
            .any(|c| c.contains("sed") || c.contains("echo")),
        "nothing may run when the repository cannot be identified"
    );
}
