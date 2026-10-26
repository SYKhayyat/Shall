use super::*;
use shall::core::executor::DryRunOutput;

#[cfg(target_os = "linux")]
#[tokio::test]
async fn a_dry_run_does_not_rebuild_the_out_of_tree_modules() {
    let mut config = Config::sandboxed(&config_dir("kernel-rebuild-dry-run"));
    config.dry_run = true;
    let (engine, mock) = engine(config).await;
    mock.set_response(
        "brew install -- linux-image-generic",
        Ok(DryRunOutput::default().into()),
    );
    let changes = install_changes(&[package("brew", "linux-image-generic")]);

    engine
        .sync(changes, GuardScope::Sync)
        .await
        .expect("the preview failed");

    let calls = mock.get_calls().await;
    assert!(
        !calls.iter().any(|call| call.contains("dkms ")),
        "the preview rebuilt the out-of-tree modules: {calls:?}"
    );
}
