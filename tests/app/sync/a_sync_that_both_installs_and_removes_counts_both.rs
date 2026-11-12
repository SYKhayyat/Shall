use super::*;
use shall::core::executor::DryRunOutput;

/// The three counters are three facts, and the existing partial-run test only ever exercised
/// two of them: it installs, it fails an install, and it removes nothing, so `removed *= 1`
/// agreed with `removed += 1` (both leave a counter that starts at zero at zero) and the
/// nightly mutation job reported the removal counter as an unpinned switch.
#[tokio::test]
async fn a_sync_that_both_installs_and_removes_counts_both() {
    let (engine, mock) = engine(Config::sandboxed(&config_dir("both-counters"))).await;

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "brew info --json=v1 -- neovim",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response(
        "brew info --json=v1 -- old-brew",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response(
        "brew info --json=v1 -- neovim",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response(
        "brew info --json=v1 -- old-brew",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response("brew install -- neovim", Ok(DryRunOutput::default().into()));
    mock.set_response(
        "brew uninstall -- old-brew",
        Ok(DryRunOutput::default().into()),
    );
    let mut changes = install_changes(&[package("brew", "neovim")]);
    changes.add_removal("brew", "old-brew");

    engine
        .sync(changes, GuardScope::Sync)
        .await
        .expect("the sync failed");

    assert_eq!(
        engine.metrics.totals(),
        (1, 1, 0),
        "one install and one removal is one of each; the two counters are separate arms of the graph walk"
    );
}
