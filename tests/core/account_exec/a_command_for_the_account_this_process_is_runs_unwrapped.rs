use super::*;

#[tokio::test]
async fn a_command_for_the_account_this_process_is_runs_unwrapped() {
    let (e, mock) = wired();
    let account = Account::current().expect("this process has an account");
    e.read_raw_as(
        &account,
        "gsettings",
        &["get", "org.gnome.desktop.interface", "gtk-theme"],
        false,
    )
    .await
    .expect("the mock answers");
    assert_eq!(
        mock.get_calls().await,
        vec!["gsettings get org.gnome.desktop.interface gtk-theme".to_string()],
        "the account this process is added a wrapper that was not there before"
    );
    let home = account.home().display().to_string();
    let env = mock.last_env.lock().await.clone();
    assert_eq!(env.get("HOME").map(String::as_str), Some(home.as_str()));
    assert_eq!(env.get("USER").map(String::as_str), Some(account.name()));
}
