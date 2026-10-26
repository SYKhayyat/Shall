use super::*;
use shall::core::executor::DryRunOutput;

#[tokio::test]
async fn a_partial_sync_counts_only_the_operations_that_happened() {
    let mut config = Config::sandboxed(&config_dir("partial-counters"));
    config.keep_going_this_run = true;
    let (engine, mock) = engine(config).await;
    mock.set_response("brew install -- neovim", Ok(DryRunOutput::default().into()));
    mock.set_response(
        "cargo install -- ripgrep",
        Err(shall::core::Error::command_failed_permanently(
            "the compiler failed",
        )),
    );
    let changes = install_changes(&[package("brew", "neovim"), package("cargo", "ripgrep")]);

    let result = engine.sync(changes, GuardScope::Sync).await;

    assert!(result.is_err(), "the failed operation did not fail the run");
    assert_eq!(
        engine.metrics.totals(),
        (1, 0, 0),
        "the counters must count the successful install and neither the failed install nor an invented removal"
    );
    let state = engine.state.lock().await;
    assert!(
        state.is_managed("brew", "neovim"),
        "the successful install was not recorded"
    );
    assert!(
        !state.is_managed("cargo", "ripgrep"),
        "the failed install was recorded as managed"
    );
}
