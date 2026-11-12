use super::*;
use shall::core::executor::DryRunOutput;

#[tokio::test]
async fn nix_flake_refs_and_queries_come_after_the_terminator() {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "nix profile install -- nixpkgs#ripgrep",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response(
        "nix search --json -- nixpkgs ripgrep",
        Ok(DryRunOutput::default().into()),
    );

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "nix profile install -- nixpkgs#ripgrep",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response(
        "nix search --json -- nixpkgs ripgrep",
        Ok(DryRunOutput::default().into()),
    );
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        Arc::new(dashmap::DashMap::new()),
    );
    let core = Arc::new(NixBackendCore::new(exec, "30d".into()));

    NixInstallable { core: core.clone() }
        .install(
            &[PackageSpec {
                name: "ripgrep".into(),
                backend: "nix".into(),
                ..Default::default()
            }],
            false,
        )
        .await
        .unwrap();
    NixSearchable { core: core.clone() }
        .search("ripgrep")
        .await
        .unwrap();

    assert_eq!(
        mock.get_calls().await,
        vec![
            "nix profile install -- nixpkgs#ripgrep",
            "nix search --json -- nixpkgs ripgrep",
        ]
    );
}
