use super::*;

#[tokio::test]
async fn flatpaks_other_name_carrying_commands_terminate_too() {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        Arc::new(dashmap::DashMap::new()),
    );
    let core = Arc::new(FlatpakBackendCore::new(exec, FLATPAK_DEFAULT_SCOPE));

    FlatpakInstallable { core: core.clone() }
        .remove(
            &["org.gimp.GIMP".to_string()],
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
        )
        .await
        .unwrap();
    FlatpakSearchable { core: core.clone() }
        .search("gimp")
        .await
        .unwrap();
    core.get_dependencies("org.gimp.GIMP").await.unwrap();

    assert_eq!(
        mock.get_calls().await,
        vec![
            "flatpak uninstall --system -y --noninteractive -- org.gimp.GIMP",
            "flatpak search -- gimp",
            "flatpak info --system --show-metadata -- org.gimp.GIMP",
        ]
    );
}
