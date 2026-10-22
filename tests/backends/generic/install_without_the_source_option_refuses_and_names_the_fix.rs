use super::*;

#[tokio::test]
async fn install_without_the_source_option_refuses_and_names_the_fix() {
    // Refusing beats guessing a URL→name mapping: the old behaviour installed happily and
    // then failed every later sync, because nothing could remove what it had installed.
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let core = Arc::new(source_option_core(mock.clone(), vfs));
    let inst = GenericInstallable { core };

    let err = inst
        .install(&[spec_with("diff", &[])], false)
        .await
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("helm:diff@url="), "{}", msg);
    assert!(
        mock.get_calls().await.is_empty(),
        "nothing may reach the machine when the declaration is incomplete"
    );
}
