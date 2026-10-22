use super::*;

/// The part that makes this a fixed *class* rather than a fixed instance: a template
/// carrying a placeholder nothing fills is refused before it runs, whatever the
/// placeholder is.
#[tokio::test]
async fn a_template_with_an_unfillable_placeholder_is_refused() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let mgr = repo_mgr("weird", mock.clone(), vfs, |c| {
        c.repo_remove_args = Some(vec!["drop".into(), "{name}".into(), "{channel}".into()]);
    });
    let err = mgr
        .remove_repo(
            "internal",
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
        )
        .await
        .expect_err("an unfilled placeholder is not a repository name")
        .to_string();
    assert!(err.contains("{channel}"), "{}", err);
    assert!(
        mock.get_calls().await.is_empty(),
        "the template ran with a placeholder in it"
    );
}
