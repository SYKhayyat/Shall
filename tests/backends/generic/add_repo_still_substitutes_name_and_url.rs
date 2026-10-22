use super::*;

/// `add_repo` substitutes both keys and always did — asserted so the guard cannot break
/// the path that was working.
#[tokio::test]
async fn add_repo_still_substitutes_name_and_url() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let mgr = repo_mgr("winget", mock.clone(), vfs, |c| {
        c.repo_add_args = Some(vec![
            "source".into(),
            "add".into(),
            "--name".into(),
            "{name}".into(),
            "--arg".into(),
            "{url}".into(),
        ]);
    });
    mgr.add_repo("internal", "https://feed.example.invalid/", false)
        .await
        .unwrap();
    assert_eq!(
        mock.get_calls().await,
        vec!["winget source add --name internal --arg https://feed.example.invalid/"]
    );
}
