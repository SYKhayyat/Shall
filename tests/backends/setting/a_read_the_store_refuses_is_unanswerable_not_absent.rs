use super::*;

/// **The caveat that ships with the ruling.** A read fails for reasons that are not "the
/// value differs": a schema `gsettings` has never heard of, a hive this account cannot
/// open. Read as *not in effect*, each would make `check` permanently red on a key nobody
/// can see and make every sync rewrite it. Only a clean read counts.
#[tokio::test]
async fn a_read_the_store_refuses_is_unanswerable_not_absent() {
    let mut a = row("probe", if cfg!(windows) { "cmd" } else { "sh" });
    a.read = if cfg!(windows) {
        vec!["cmd".into(), "/C".into(), "exit 1".into()]
    } else {
        vec!["sh".into(), "-c".into(), "exit 1".into()]
    };
    let core = SettingBackendCore::new(CommandExecutor::new(false, false), vec![a]);
    assert_eq!(
        core.holds("org.gnome.x/k", "anything", None, None).await,
        None,
        "a refused read must not be reported as a value that differs"
    );
}
