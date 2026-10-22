use super::*;

#[tokio::test]
async fn install_from_a_source_option_sends_the_source_and_removes_by_name() {
    // U39. The whole bug in one test: what goes out at install is the URL, what goes out
    // at remove is the name, and they come from the same one-line declaration.
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let core = Arc::new(source_option_core(mock.clone(), vfs));
    let inst = GenericInstallable { core };

    let url = "https://github.com/databus23/helm-diff";
    inst.install(&[spec_with("diff", &[("url", url)])], false)
        .await
        .unwrap();
    inst.remove(
        &["diff".to_string()],
        false,
        crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
    )
    .await
    .unwrap();

    let calls = mock.get_calls().await;
    assert!(
        calls
            .iter()
            .any(|c| c.contains("plugin install") && c.contains(url)),
        "install must send the url: {:?}",
        calls
    );
    assert!(
        !calls
            .iter()
            .any(|c| c.contains("plugin install") && c.contains(" diff")),
        "install must not send the name: {:?}",
        calls
    );
    assert!(
        calls
            .iter()
            .any(|c| c.contains("plugin uninstall") && c.contains("diff")),
        "remove must send the name: {:?}",
        calls
    );
}
