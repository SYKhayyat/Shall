use super::*;

/// The link that turned an executor detail into a wrong answer about the machine.
///
/// `run_output` handed back `Ok("")` for a lister that died without a word, the parser
/// found no packages in the empty string, and `list_installed` reported `Ok(vec![])` — a
/// manager with nothing installed. Nothing in the chain thought anything had failed.
/// Measured on winget: 1 run in 16 under concurrent cold start, and `shall list --backend
/// winget` printed nothing and exited 0 with 280 packages on the machine.
#[tokio::test]
async fn a_lister_that_died_silently_is_a_failure_not_an_empty_machine() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_response(
        "dpkg-query -W -f=${db:Status-Status} ${Package} ${Version}\\n",
        Ok(shall::core::executor::silent_failure(1)),
    );
    let q = queryable_with(ManualListing::AllInstalled, mock.clone(), vfs);

    let err = q.list_installed().await.expect_err(
        "a lister that exited non-zero without a word must not report an empty machine",
    );
    assert!(
        err.to_string().contains("no output"),
        "the failure must say the lister produced nothing: {err}"
    );
}
