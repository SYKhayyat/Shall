use super::*;

#[tokio::test]
async fn unsupported_backend_reports_nothing_rather_than_everything() {
    // The safety backstop: a manager with dependencies and no way to name the user's
    // choices must return an empty list AND admit it via tracks_manual, so adoption
    // skips it. Returning list_installed here is a confident wrong answer.
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_response(
        "dpkg-query -W -f=${db:Status-Status} ${Package} ${Version}\\n",
        Ok(DryRunOutput {
            stdout: b"installed apt 2.7.14\ninstalled libperl5.38t64 5.38.2\n".to_vec(),
            stderr: vec![],
        }
        .into()),
    );

    let q = queryable_with(ManualListing::Unsupported, mock, vfs);
    assert!(!q.tracks_manual());
    assert!(
        q.list_manual().await.unwrap().is_empty(),
        "adopting nothing is safe; adopting everything is catastrophic"
    );
    // list_installed still works — only the *intent* question is unanswerable.
    assert_eq!(q.list_installed().await.unwrap().len(), 2);
}
