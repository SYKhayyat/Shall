use super::*;

/// **And when nothing upgrades, that is not success.** The batch failed, every individual
/// retry failed, and the function returned `Ok(())` — exit 0 with zero packages upgraded,
/// `ensure_status` logged at debug. The isolating loop exists so one bad package does not
/// strand the rest; it does not exist to swallow the report when they all fail.
#[tokio::test]
async fn an_upgrade_whose_every_attempt_fails_is_reported() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    // Every shape the reinstall takes must fail: the batch and each individual retry.
    // (The mock answers by exact command line; npm terminates its options, so the
    // names arrive after `--`.)
    for line in [
        "npm install -g -- a b",
        "npm install -g -- a",
        "npm install -g -- b",
    ] {
        mock.set_response(
            line,
            Err(shall::core::Error::Io("registry unreachable".into())),
        );
    }
    let mut core = apt_like_core(mock.clone(), vfs);
    core.name = "npm".into();
    core.config.name = "npm".into();
    core.config.list_binary = None;
    core.config.list_args = vec!["ls".into()];
    core.config.install_args = vec!["install".into(), "-g".into()];
    core.config.upgrade_reinstall_args = Some(vec!["install".into(), "-g".into()]);
    core.parser = Arc::new(shall::parsers::LambdaParser {
        installed_fn: |_| Ok(vec![Package::new("a", "npm"), Package::new("b", "npm")]),
        search_fn: |_| Vec::new(),
    });
    let u = GenericUpgradable {
        core: Arc::new(core),
    };
    let e = u
        .upgrade(false)
        .await
        .expect_err("nothing upgraded must not read as upgraded");
    assert!(e.to_string().contains("failed"), "{e}");
}
