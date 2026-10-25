use super::*;

/// Both halves of what the `tools` image measured on 2026-07-29, in one place because they
/// are one lifecycle: a mix archive that cannot be pinned cannot be installed at all on an
/// older Elixir, and a removal without `--force` reports success and removes nothing.
///
/// ```text
/// $ mix archive.install hex --force phx_new          -> supports only Elixir ~> 1.17 (exit 1)
/// $ mix archive.install hex --force phx_new 1.6.16   -> creating /root/.mix/archives/phx_new-1.6.16
/// $ mix archive.uninstall phx_new  </dev/null        -> `Are you sure…? [Yn]`, exit 0, STILL INSTALLED
/// $ mix archive.uninstall --force -- phx_new         -> gone
/// ```
///
/// The option terminator is Shall's, and it was measured rather than assumed: both of the
/// commands above were run in that exact shape, because two managers in this tree turned
/// out to read `--` as a package name (B25) and mix does not.
/// ```
#[tokio::test]
async fn a_mix_archive_is_pinnable_and_its_removal_does_not_wait_for_an_answer() {
    use dashmap::DashMap;
    use shall::core::executor::MockExecutor;
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let exec =
        CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    let mut reg = BackendRegistry::new();
    register_mix(&mut reg, &exec);
    let mix = reg.get("mix").expect("mix is registered");
    let inst = mix.as_installable().expect("installs");

    inst.install(
        &[shall::core::PackageSpec {
            name: "phx_new".into(),
            backend: "mix".into(),
            options: [("version", "1.6.16")].into_iter().collect(),
            ..Default::default()
        }],
        false,
    )
    .await
    .unwrap();
    inst.remove(&["phx_new".to_string()], false, removal_token().await)
        .await
        .unwrap();

    let calls = mock.get_calls().await;
    // **Behind the terminator, both of them.** mix's version is a bare operand, so `--`
    // protects it exactly as it protects the name; the pin used to be labelled a *flag*,
    // which gave the terminator up on every pinned install and kept it on every unpinned
    // one. Measured, `tools` image 2026-08-04: `mix archive.install hex --force --
    // <name> <version>` is identical to the same line without the terminator (Q30).
    assert!(
        calls
            .iter()
            .any(|c| c == "mix archive.install hex --force -- phx_new 1.6.16"),
        "the pinned version never reached mix: {:?}",
        calls
    );
    assert!(
        calls
            .iter()
            .any(|c| c == "mix archive.uninstall --force -- phx_new"),
        "the removal would sit on a prompt and report success: {:?}",
        calls
    );
}
