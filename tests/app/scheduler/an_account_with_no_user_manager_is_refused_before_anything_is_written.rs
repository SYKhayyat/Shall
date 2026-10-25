use super::*;

#[tokio::test]
async fn an_account_with_no_user_manager_is_refused_before_anything_is_written() {
    use dashmap::DashMap;
    use shall::core::executor::MockExecutor;
    use std::sync::Arc;

    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_response(
        "systemctl --no-pager --user daemon-reload",
        Err(Error::CommandFailed {
            message: "Failed to connect to bus: No medium found".into(),
            retry: shall::core::Retryability::Permanent,
            absent_name: false,
        }),
    );
    let exec =
        CommandExecutor::with_layer(false, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    let account = shall::core::account::Account::current().unwrap();
    let target = me();
    let err = LinuxSystemdProvisioner
        .add_task(
            &exec,
            &cfg("0 2 * * *"),
            Path::new("/usr/local/bin/shall"),
            &target,
        )
        .await
        .expect_err("no user manager must be refused");
    let err = err.to_string();
    assert!(
        err.contains(account.name()),
        "the refusal names no account: {err}"
    );
    assert!(
        err.contains("user manager"),
        "the refusal does not say what is missing: {err}"
    );
    assert!(err.contains("Nothing was written"), "{err}");

    let calls = mock.get_calls().await;
    assert_eq!(calls.len(), 1, "a refused schedule ran {calls:?}");
    assert!(calls[0].contains("daemon-reload"), "{calls:?}");
}
