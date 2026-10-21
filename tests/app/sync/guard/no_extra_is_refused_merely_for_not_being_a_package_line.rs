use super::*;

#[tokio::test]
async fn no_extra_is_refused_merely_for_not_being_a_package_line() {
    // The trap this kind exists to avoid: `protection_of`'s declarability test asks whether
    // a package line could hold the name, and no extras key can â€” `link:/home/u/.vimrc` is
    // not a package line and never parses as one. Running that test over extras marks all
    // six kinds `Undeclarable` and refuses every teardown on every machine forever, which
    // is a guard that has stopped being about what the user asked for.
    let reg = Arc::new(BackendRegistry::new());
    let cfg = config_with(20);
    let all_six = extras(&[
        "link:/home/u/.vimrc",
        "service:nginx",
        "setting:org.gnome.desktop.interface color-scheme",
        "shim:rg",
        "schedule:nightly-sync",
        "repo:apt:ppa:x/y",
    ]);
    let report = inspect_removals(
        &cfg,
        &reg,
        &all_six,
        RemovalKind::Extra,
        &Reaping::new(),
        &mut EssentialsCache::default(),
    )
    .await;
    assert!(
        report.is_empty(),
        "an ordinary teardown of one of each kind must be allowed: {:?}",
        report.objections
    );
}
