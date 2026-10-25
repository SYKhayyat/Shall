use super::*;

#[cfg(unix)]
#[tokio::test]
async fn a_command_for_another_account_is_refused_where_this_process_cannot_become_it() {
    if CommandExecutor::is_root() {
        return;
    }
    let (e, mock) = wired();
    let account = another_account();
    let err = e
        .read_raw_as(&account, "gsettings", &["get", "x"], false)
        .await
        .expect_err("a refused account must not run the command");
    let message = err.to_string();
    assert!(message.contains(account.name()), "{message}");
    assert!(mock.get_calls().await.is_empty(), "{message}");
}
