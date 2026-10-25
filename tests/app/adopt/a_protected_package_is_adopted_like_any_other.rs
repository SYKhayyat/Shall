use super::*;

#[tokio::test]
async fn a_protected_package_is_adopted_like_any_other() {
    // E7/II.9. This test used to assert the opposite — that `protected_packages` kept a
    // package out of the manifest — which made "protected" mean two contradictory
    // things: *never remove* in the guard, *never adopt* here. A package you could not
    // adopt and could not remove, for the same reason.
    //
    // Protection means one thing: never remove. `python3` belongs in your file like
    // everything else you chose, and deleting that line is what the guard refuses
    // (V.26).
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs));
    mock.set_response(
        "apt-mark showmanual",
        Ok(DryRunOutput {
            stdout: b"jq\npython3\n".to_vec(),
            stderr: vec![],
        }
        .into()),
    );
    let reg = registry_with(
        ManualListing::Command {
            binary: Some("apt-mark".into()),
            args: vec!["showmanual".into()],
            format: ManualFormat::BareNames,
        },
        mock,
    );
    let config = Config {
        guard: shall::config::GuardSettings {
            protected_packages: vec!["python3".into()],
            ..Default::default()
        },
        ..Config::default()
    };
    let m = Adopter::new(
        over(reg),
        Arc::new(Mutex::new(StateRegistry::default())),
        &config,
    );
    let names: Vec<String> = m
        .audit()
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert_eq!(
        names,
        vec!["jq", "python3"],
        "adopt must take a protected package: protection stops its REMOVAL, not its \
         adoption"
    );
}
