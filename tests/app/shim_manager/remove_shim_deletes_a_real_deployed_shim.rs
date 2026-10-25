use super::*;

#[tokio::test]
async fn remove_shim_deletes_a_real_deployed_shim() {
    let tmp = tempdir().unwrap();
    let bin = tmp.path().join("bin");
    let mgr = ShimManager::with_bin_dir(bin.clone()).await.unwrap();

    // A shim is the shall binary under another name. The test binary stands in for it:
    // `is_deployed_shim` compares against `current_exe`, which here is the test runner.
    // Windows shims carry `.exe`, which is the name `remove_shim` will look for.
    let exe = std::env::current_exe().unwrap();
    let shim = bin.join(if cfg!(windows) {
        "ripgrep.exe"
    } else {
        "ripgrep"
    });
    tokio::fs::copy(&exe, &shim).await.unwrap();

    mgr.remove_shim(
        "ripgrep",
        crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
    )
    .await
    .unwrap();

    assert!(!shim.exists(), "a real shim must still be removable");
}
