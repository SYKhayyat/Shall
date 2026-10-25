use super::*;

#[tokio::test]
async fn a_read_is_asked_in_the_account_the_line_named() {
    let account = shall::core::account::Account::current().unwrap();
    let read = if cfg!(windows) {
        vec!["cmd".into(), "/C".into(), "echo %USERNAME%".into()]
    } else {
        vec!["sh".into(), "-c".into(), "printf '%s' \"$USER\"".into()]
    };
    let mut a = row("probe", if cfg!(windows) { "cmd" } else { "sh" });
    a.read = read;
    let core = SettingBackendCore::new(CommandExecutor::new(false, false), vec![a]);

    let mine = core
        .holds("org.gnome.x/k", account.name(), None, Some(account.name()))
        .await;
    assert_eq!(
        mine,
        Some(true),
        "a read for the account the line named did not answer from that account"
    );
    assert_eq!(
        core.holds("org.gnome.x/k", account.name(), None, None)
            .await,
        Some(true),
        "the unscoped read and the self-named read disagree about the same account"
    );
    assert_eq!(
        core.holds("org.gnome.x/k", "not-the-value", None, Some(account.name()))
            .await,
        Some(false),
        "a read that succeeded reported no comparison rather than a difference"
    );
}
