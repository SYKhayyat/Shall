use super::*;
use shall::core::executor::DryRunOutput;

#[cfg(target_os = "linux")]
#[tokio::test]
async fn a_module_that_did_not_build_fails_the_sync_loudly() {
    let (engine, mock) = engine(Config::sandboxed(&config_dir("kernel-rebuild-fail"))).await;
    mock.set_response(
        "sudo -n dkms autoinstall",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response(
        "dkms status",
        Ok(DryRunOutput {
            stdout: b"nvidia/550.78, 6.8.0-31-generic, x86_64: added\n".to_vec(),
            stderr: Vec::new(),
        }
        .into()),
    );
    let changes = install_changes(&[package("brew", "linux-image-generic")]);

    let error = engine
        .sync(changes, GuardScope::Sync)
        .await
        .expect_err("an unbuilt module did not fail the sync");

    let message = error.to_string();
    assert!(
        message.contains("did not build") && message.contains("nvidia/550.78"),
        "the failure does not name the module that did not build: {message}"
    );
}
