use super::*;

/// Deploy and remove must agree about where a shim lives, for every name, or removal
/// reports success over a shim that is still on PATH.
///
/// They did not. `create_shim` replaced any extension that was not `exe`; `remove_shim`
/// appended one only when there was none. So `shim:tool.bat` deployed `tool.exe` and
/// removal looked for `tool.bat`, found nothing, and returned `Ok`. Found while giving the
/// path rule one definition so drift-reporting could ask about it — the two copies had
/// already diverged.
#[tokio::test]
async fn deploy_and_remove_agree_on_where_a_shim_lives() {
    for name in ["tool", "tool.bat", "tool.cmd", "tool.exe"] {
        let tmp = tempdir().unwrap();
        let bin = tmp.path().join("bin");
        let mgr = ShimManager::with_bin_dir(bin.clone()).await.unwrap();

        mgr.create_shim(name).await.unwrap();
        let deployed = mgr.shim_path(name);
        assert!(
            deployed.exists(),
            "`{name}` was not deployed where `shim_path` says it goes: {deployed:?}"
        );
        assert!(
            mgr.is_in_effect(name).await,
            "`{name}` is deployed and does not report itself in effect, so drift \
             reporting would call a placed shim missing"
        );

        mgr.remove_shim(
            name,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
        )
        .await
        .unwrap();
        assert!(
            !deployed.exists(),
            "`{name}` survived removal at {deployed:?} — removal looked somewhere else and \
             returned Ok"
        );
        assert!(!mgr.is_in_effect(name).await);
    }
}
