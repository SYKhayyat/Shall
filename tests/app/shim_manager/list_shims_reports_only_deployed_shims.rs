use super::*;

#[tokio::test]
async fn list_shims_reports_only_deployed_shims() {
    let tmp = tempdir().unwrap();
    let bin = tmp.path().join("bin");
    let mgr = ShimManager::with_bin_dir(bin.clone()).await.unwrap();

    let exe = std::env::current_exe().unwrap();
    let deployed = bin.join(if cfg!(windows) {
        "ripgrep.exe"
    } else {
        "ripgrep"
    });
    tokio::fs::copy(&exe, &deployed).await.unwrap();
    tokio::fs::write(bin.join("my-script"), b"#!/bin/sh\n")
        .await
        .unwrap();

    let shims = mgr.list_shims().await.unwrap();

    assert!(shims.iter().any(|s| s.starts_with("ripgrep")));
    assert!(
        !shims.iter().any(|s| s.starts_with("my-script")),
        "a file Shall never deployed is not a shim: {:?}",
        shims
    );
}
