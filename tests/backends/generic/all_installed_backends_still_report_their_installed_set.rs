use super::*;

#[tokio::test]
async fn all_installed_backends_still_report_their_installed_set() {
    // winget/choco/mas install no dependencies, so everything listed was asked for.
    // The Unsupported default must not silently swallow these.
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_response(
        "dpkg-query -W -f=${db:Status-Status} ${Package} ${Version}\\n",
        Ok(DryRunOutput {
            stdout: b"installed jq 1.7.1\n".to_vec(),
            stderr: vec![],
        }
        .into()),
    );
    let q = queryable_with(ManualListing::AllInstalled, mock, vfs);
    assert!(q.tracks_manual());
    assert_eq!(q.list_manual().await.unwrap().len(), 1);
}
