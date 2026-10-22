use super::*;

#[tokio::test]
async fn an_empty_source_option_is_as_missing_as_no_option() {
    // `@url=` with nothing after it would otherwise run `helm plugin install ''`.
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let core = Arc::new(source_option_core(mock.clone(), vfs));
    let inst = GenericInstallable { core };
    assert!(inst
        .install(&[spec_with("diff", &[("url", "  ")])], false)
        .await
        .is_err());
}
