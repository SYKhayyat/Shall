use super::*;

/// The boundary the fix must not cross. A manager with genuinely nothing installed exits
/// 0 and says nothing, and that empty listing is a real answer — turning *it* into an
/// error would break every clean machine.
#[tokio::test]
async fn a_lister_that_succeeded_with_nothing_to_say_still_reports_an_empty_machine() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_response(
        "dpkg-query -W -f=${db:Status-Status} ${Package} ${Version}\\n",
        Ok(DryRunOutput {
            stdout: vec![],
            stderr: vec![],
        }
        .into()),
    );
    let q = queryable_with(ManualListing::AllInstalled, mock.clone(), vfs);
    assert!(
        q.list_installed()
            .await
            .expect("exit 0 is an answer")
            .is_empty(),
        "an empty listing at exit 0 is a machine with nothing installed, not a failure"
    );
}
