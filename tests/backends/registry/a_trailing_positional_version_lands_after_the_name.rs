use super::*;

/// A pinned version rides where that manager puts it, and still behind the terminator.
///
/// The argv table drives one declaration per backend and that declaration is unpinned, so
/// the pinned shape has no row. The since-deleted `pubdart.rs` asserted it before it became
/// data and this is
/// that assertion, kept rather than lost with the module: `dart pub global activate --
/// webdev 2.7.0` is a trailing positional, not `webdev@2.7.0`, and the two are one
/// `VersionPin` variant apart.
#[tokio::test]
async fn a_trailing_positional_version_lands_after_the_name() {
    use dashmap::DashMap;
    use shall::core::executor::MockExecutor;

    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let exec =
        CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    let mut reg = BackendRegistry::new();
    register_pubdart(&mut reg, &exec);

    let inst = reg.get("pub").unwrap().as_installable().unwrap().clone();
    let mut spec = shall::core::PackageSpec {
        name: "webdev".into(),
        backend: "pub".into(),
        ..Default::default()
    };
    spec.options.set("version", "2.7.0");
    let _ = inst.install(&[spec], false).await;

    let calls = mock.get_calls().await;
    assert!(
        calls
            .iter()
            .any(|c| c.contains("dart pub global activate -- webdev 2.7.0")),
        "the pinned version did not land as a trailing positional: {calls:?}"
    );
}
