use super::*;

/// Q45. **One `nix profile install` for the batch, and one `remove` for the by-name path.**
///
/// Each invocation evaluates nixpkgs and cuts a new profile generation, so N one at a time
/// is N evaluations and N generations for one change the user made.
///
/// The by-name removal was verified against real nix 2.x in a container before it was
/// batched — `nix profile remove hello ripgrep` reported `removed 2 packages, kept 17` and
/// left the one it was not given. The *indexed* path stays one at a time on purpose: its
/// highest-index-first ordering exists because positional indices renumber, and no nix that
/// still reports them was available to prove a batched form safe.
#[tokio::test]
async fn a_batch_of_installables_is_one_nix_call() {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        Arc::new(dashmap::DashMap::new()),
    );
    let core = Arc::new(NixBackendCore::new(exec, "30d".to_string()));
    let specs = vec![
        shall::core::PackageSpec {
            name: "hello".into(),
            backend: "nix".into(),
            ..Default::default()
        },
        shall::core::PackageSpec {
            name: "ripgrep".into(),
            backend: "nix".into(),
            ..Default::default()
        },
    ];
    NixInstallable { core: core.clone() }
        .install(&specs, false)
        .await
        .unwrap();

    let calls = mock.get_calls().await;
    assert_eq!(calls.len(), 1, "one install for the batch, got {:?}", calls);
    assert!(calls[0].contains("nixpkgs#hello"), "{:?}", calls);
    assert!(calls[0].contains("nixpkgs#ripgrep"), "{:?}", calls);
}
