use super::*;

/// **`#164`: the setting is read by the unattended run too.**
///
/// `never` is an answer, and `watch` is the spelling of `sync` that runs with nobody watching — so
/// it is the one that most needs the answer. It passed `confirm: false`, which skipped the whole
/// bare-declaration block: the daemon installed exactly what its owner's configuration forbade and
/// reported `applied 1 change(s)` with nothing saying a policy had been crossed.
#[tokio::test]
async fn the_setting_is_read_by_the_unattended_run_too() {
    let kernel = declaring(&["dos2unix"], shall::config::config::BareInstall::Never).await;

    let done = watch_reconcile(&kernel.app)
        .await
        .expect("`watch` reconciles, and a decline by policy is not a reconcile that failed");

    assert_eq!(
        done.applied, 0,
        "the daemon applied a change its configuration declines"
    );
    assert_eq!(done.not_installed, 1);
    assert_eq!(
        done.declined_by_policy, 1,
        "the decline must be attributed to the setting here too, or `watch` reports it as work \
         that could not be done"
    );
    let calls = kernel.mock_executor.get_calls().await;
    assert!(
        !calls.iter().any(|c| c.contains("install")),
        "`watch` reached the machine with the package `never` forbids: {calls:?}"
    );
}

/// **The control, because "the daemon did nothing" is also what a broken fixture looks like.** One
/// setting apart, the same fixture is installed by the same call.
#[tokio::test]
async fn the_unattended_run_still_installs_when_nothing_forbids_it() {
    let kernel = declaring(&["dos2unix"], shall::config::config::BareInstall::Always).await;

    let done = watch_reconcile(&kernel.app)
        .await
        .expect("an `always` machine installs what its files select");

    assert_eq!(done.declined_by_policy, 0);
    assert_eq!(done.applied, 1, "the control never applied anything");
    let calls = kernel.mock_executor.get_calls().await;
    assert!(
        calls
            .iter()
            .any(|c| c.contains("dos2unix") && c.contains("install")),
        "the control never reached the manager: {calls:?}"
    );
}
