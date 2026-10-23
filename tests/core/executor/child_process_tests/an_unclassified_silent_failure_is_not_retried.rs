use super::*;

/// The other side: a failure the policy does **not** classify is asked once and reported.
/// Retrying everything would turn a manager that is simply broken into a slow one.
#[tokio::test]
async fn an_unclassified_silent_failure_is_not_retried() {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let e = CommandExecutor::with_layer(false, false, mock.clone(), vfs, Arc::new(DashMap::new()))
        .with_exit_policy(shall::core::exit_policy::winget());
    mock.set_response("winget list", Ok(super::silent_failure(1)));

    let _ = e.run_output("winget", &["list"], false).await.unwrap_err();
    let tries = mock
        .get_calls()
        .await
        .iter()
        .filter(|c| c.as_str() == "winget list")
        .count();
    assert_eq!(
        tries, 1,
        "an unclassified failure must be asked once, not {tries} times"
    );
}
