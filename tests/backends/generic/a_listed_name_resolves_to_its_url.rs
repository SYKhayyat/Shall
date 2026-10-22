use super::*;

/// The other `{url}` template: gem removes by source URL, and a name the listing knows
/// resolves to one.
#[tokio::test]
async fn a_listed_name_resolves_to_its_url() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let mut listed: std::process::Output = DryRunOutput::new().into();
    listed.stdout = b"internal https://gems.example.invalid/\n".to_vec();
    mock.set_response("gem sources", Ok(listed));
    let mgr = repo_mgr("gem", mock.clone(), vfs, |c| {
        c.repo_remove_args = Some(vec!["sources".into(), "-r".into(), "{url}".into()]);
        c.repo_list_args = Some(vec!["sources".into()]);
    });
    mgr.remove_repo(
        "internal",
        false,
        crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
    )
    .await
    .unwrap();
    let calls = mock.get_calls().await;
    assert!(
        calls
            .iter()
            .any(|c| c == "gem sources -r https://gems.example.invalid/"),
        "{:?}",
        calls
    );
}
