use super::*;

#[tokio::test]
async fn a_teardown_of_a_contradictory_row_is_refused_before_it_resets_anything() {
    let core = Arc::new(SettingBackendCore::new(
        CommandExecutor::new(false, false),
        adapters(vec![registry_like()]),
    ));
    let installable = SettingInstallable { core: core.clone() };
    let err = installable
        .remove(
            &["org.gnome.x/theme@scope=system@user=alice".to_string()],
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Sync).await,
        )
        .await
        .expect_err("a row that was never written must not be reset");
    let message = err.to_string();
    assert!(
        message.contains("nothing to write that is both"),
        "{message}"
    );
    assert!(message.contains("alice"), "{message}");
}
