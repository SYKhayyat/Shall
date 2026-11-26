use super::*;

/// **The state a cancelled rebuild leaves behind is the state the skip reads as converged.**
///
/// Measured in the NixOS VM leg (`#225`): `nixos-rebuild switch` ran past its budget, the file and
/// the import stayed exactly as Shall had written them, the transaction retried, the retry saw a
/// file that matched what it would write and returned `Ok(())` without running anything — and the
/// transaction reported `SUCCESS` with `(Retries: 1)` over a system that had not changed.
///
/// The file agreeing with itself is evidence about the file. Only a package change forces the
/// question "does the machine have it", and that question is answered by rebuilding.
fn configured(cfg: &Path, module: &Module) {
    std::fs::create_dir_all(cfg).unwrap();
    std::fs::write(cfg.join(GENERATED), render(module).unwrap()).unwrap();
    std::fs::write(
        cfg.join("configuration.nix"),
        format!("{{ ... }}: {{ imports = [ ./{} ]; }}\n", GENERATED),
    )
    .unwrap();
}

fn core_for(cfg: &Path) -> (Arc<NixosBackendCore>, Arc<MockExecutor>) {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs.clone(),
        Arc::new(DashMap::new()),
    );
    (
        Arc::new(NixosBackendCore::new(exec, cfg.to_path_buf(), true)),
        mock,
    )
}

#[tokio::test]
async fn a_package_install_rebuilds_even_when_the_file_already_matches() {
    let dir = tempfile::tempdir().unwrap();
    let module = packages(&["hello"]);
    configured(dir.path(), &module);

    let (core, mock) = core_for(dir.path());

    let installable = NixosInstallable { core: core.clone() };
    let spec = PackageSpec {
        name: "hello".to_string(),
        ..Default::default()
    };
    installable
        .install(&[spec], false)
        .await
        .expect("the install succeeds");

    // Nothing is registered, so every command that ran is recorded verbatim — and the assertion
    // reads the argv rather than a stub, which keeps it independent of whether this host prefixes
    // the command with `sudo`.
    let ran = mock.unstubbed_registrations();
    assert!(
        ran.iter()
            .any(|c| c.contains("nixos-rebuild") && c.contains("switch")),
        "a package install must rebuild the system, even when the file already says what it \
         would write; commands that ran were {ran:?}"
    );
}

#[tokio::test]
async fn a_reprojected_state_still_skips_the_rebuild_it_did_not_change() {
    let dir = tempfile::tempdir().unwrap();
    let module = packages(&["hello"]);
    configured(dir.path(), &module);

    let (core, mock) = core_for(dir.path());

    core.write_and_switch(&module, Switch::UnlessConverged)
        .await
        .expect("a converged file needs no rebuild");

    assert_eq!(
        std::fs::read_to_string(dir.path().join(GENERATED)).unwrap(),
        render(&module).unwrap(),
        "the file this run would have written is the file that was already there"
    );
    let ran = mock.unstubbed_registrations();
    assert!(
        !ran.iter().any(|c| c.contains("nixos-rebuild")),
        "a state re-projected without a change must not spend a rebuild: {ran:?}"
    );
}
