use super::*;

/// The finding: `{url}` was never substituted on the removal path, so `sed` searched for
/// the literal text `{url}`, matched nothing, and **exited 0** — Shall reported a
/// repository removed that was still in the file.
#[tokio::test]
async fn apk_repo_removal_carries_the_real_url_and_no_placeholder() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let mgr = apk_repo(mock.clone(), vfs);
    mgr.remove_repo(
        "https://dl-cdn.alpinelinux.org/alpine/edge/testing",
        false,
        crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
    )
    .await
    .expect("a URL is a repository apk can be told to forget");
    let calls = mock.get_calls().await;
    assert!(
        calls.iter().any(|c| c.contains("dl-cdn.alpinelinux.org")),
        "{:?}",
        calls
    );
    assert!(
        !calls.iter().any(|c| c.contains("{url}")),
        "the placeholder reached the machine: {:?}",
        calls
    );
}
