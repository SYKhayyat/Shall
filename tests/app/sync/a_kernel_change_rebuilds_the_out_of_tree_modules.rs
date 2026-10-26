use super::*;
use shall::core::executor::DryRunOutput;

#[cfg(target_os = "linux")]
#[tokio::test]
async fn a_kernel_change_rebuilds_the_out_of_tree_modules() {
    let (engine, mock) = engine(Config::sandboxed(&config_dir("kernel-rebuild"))).await;
    mock.set_response(
        "sudo -n dkms autoinstall",
        Ok(DryRunOutput::default().into()),
    );
    mock.set_response(
        "dkms status",
        Ok(DryRunOutput {
            stdout: b"nvidia/550.78, 6.8.0-31-generic, x86_64: installed\n".to_vec(),
            stderr: Vec::new(),
        }
        .into()),
    );
    let changes = install_changes(&[package("brew", "linux-image-generic")]);

    engine
        .sync(changes, GuardScope::Sync)
        .await
        .expect("the kernel sync failed");

    let calls = mock.get_calls().await;
    assert!(
        calls.iter().any(|call| call == "sudo -n dkms autoinstall"),
        "the kernel change did not rebuild the out-of-tree modules: {calls:?}"
    );
    assert!(
        calls.iter().any(|call| call == "dkms status"),
        "the rebuild did not read the module status: {calls:?}"
    );
}
