use super::*;

/// asdf's fallback is an operand too, and used to be treated as a flag.
///
/// The unpinned branch set "there is a trailing option" unconditionally, so `latest` — a
/// bare word — suppressed the terminator by a rule meant for `-v`. asdf still gets no `--`,
/// because the terminator table measured it (`No such plugin: --`); what changed is that the
/// two layers now say so for two correct reasons instead of agreeing by luck.
#[tokio::test]
async fn a_required_version_fallback_is_an_operand_not_a_flag() {
    use dashmap::DashMap;
    use shall::core::executor::MockExecutor;

    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let exec =
        CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    let mut reg = BackendRegistry::new();
    register_asdf(&mut reg, &exec);
    let inst = reg.get("asdf").unwrap().as_installable().unwrap().clone();

    let _ = inst
        .install(
            &[shall::core::PackageSpec {
                name: "nodejs".into(),
                backend: "asdf".into(),
                ..Default::default()
            }],
            false,
        )
        .await;

    let calls = mock.get_calls().await;
    assert!(
        calls.iter().any(|c| c == "asdf install nodejs latest"),
        "an unpinned asdf line must ask for `latest`, with no terminator: {calls:?}"
    );
}
