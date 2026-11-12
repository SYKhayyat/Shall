use super::*;
use shall::core::executor::DryRunOutput;

#[tokio::test]
async fn a_deliberate_purge_is_not_refused_by_the_accidental_removal_ceiling() {
    let mut config = Config::sandboxed(&config_dir("deliberate-purge"));
    config.guard.max_removals = 1;
    let (engine, mock) = engine(config).await;

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "brew info --json=v1 -- old-brew",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response("cargo install --list", Ok(DryRunOutput::default().into()));
    mock.set_response(
        "brew info --json=v1 -- old-brew",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response("cargo install --list", Ok(DryRunOutput::default().into()));
    mock.set_response(
        "brew uninstall -- old-brew",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response(
        "cargo uninstall -- old-cargo",
        Ok(DryRunOutput::default().into()),
    );
    let changes = removal_changes(&[("brew", "old-brew"), ("cargo", "old-cargo")]);

    engine
        .sync(changes, GuardScope::PurgeUndeclared)
        .await
        .expect("the deliberate purge was refused");

    let calls = mock.get_calls().await;
    assert!(
        calls
            .iter()
            .any(|call| call == "brew uninstall -- old-brew"),
        "the brew package was not removed: {calls:?}"
    );
    assert!(
        calls
            .iter()
            .any(|call| call == "cargo uninstall -- old-cargo"),
        "the cargo package was not removed: {calls:?}"
    );
}
