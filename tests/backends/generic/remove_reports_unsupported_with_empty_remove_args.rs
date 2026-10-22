use super::*;

#[tokio::test]
async fn remove_reports_unsupported_with_empty_remove_args() {
    // A manager with no uninstall verb encodes it as empty remove_args → Unsupported,
    // rather than running the bare binary against the package names.
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let core = Arc::new(apt_like_core(mock, vfs)); // apt_like_core sets remove_args: vec![]
    let inst = GenericInstallable { core };
    match inst
        .remove(
            &["ghc".to_string()],
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
        )
        .await
    {
        Err(shall::core::Error::Unsupported(name)) => assert_eq!(name, "apt"),
        other => panic!("expected Unsupported, got {:?}", other),
    }
}
