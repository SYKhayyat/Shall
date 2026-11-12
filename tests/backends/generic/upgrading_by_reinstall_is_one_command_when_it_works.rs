use super::*;
use shall::core::executor::DryRunOutput;

/// `Q46`: **upgrading by reinstalling is one command, and still isolates a bad package.**
///
/// A manager with no upgrade-all verb upgrades by re-installing what it has. That ran one
/// `npm install -g <name>` per package — forty global packages, forty resolutions, forty
/// registry conversations. Batching alone would have thrown away the reason the loop
/// existed: one package that will not reinstall must not stop the other thirty-nine. So the
/// batch is the fast path and the loop is the recovery, which costs nothing when everything
/// works and loses nothing when it does not.
#[tokio::test]
async fn upgrading_by_reinstall_is_one_command_when_it_works() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "npm install -g -- a b c",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response("npm ls", Ok(DryRunOutput::default().into()));
    let mut core = apt_like_core(mock.clone(), vfs);
    core.name = "npm".into();
    core.config.name = "npm".into();
    core.config.list_binary = None;
    core.config.list_args = vec!["ls".into()];
    core.config.install_args = vec!["install".into(), "-g".into()];
    core.config.upgrade_reinstall_args = Some(vec!["install".into(), "-g".into()]);
    core.parser = Arc::new(shall::parsers::LambdaParser {
        installed_fn: |_| {
            Ok(vec![
                Package::new("a", "npm"),
                Package::new("b", "npm"),
                Package::new("c", "npm"),
            ])
        },
        search_fn: |_| Vec::new(),
    });
    let u = GenericUpgradable {
        core: Arc::new(core),
    };
    u.upgrade(false).await.unwrap();

    let installs: Vec<String> = mock
        .get_calls()
        .await
        .into_iter()
        .filter(|c| c.contains("install"))
        .collect();
    assert_eq!(
        installs.len(),
        1,
        "three packages must be one command, got {:?}",
        installs
    );
    for name in ["a", "b", "c"] {
        assert!(installs[0].contains(name), "{:?}", installs);
    }
}
