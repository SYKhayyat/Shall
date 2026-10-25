use super::*;

/// `bin_dir` is `~/.local/bin`, shared with the user and every other tool. Removal
/// used to match on FILENAME alone, so a managed package named `jq` made every sync
/// delete whatever `~/.local/bin/jq` happened to be. The ownership test is what stands
/// between a teardown and a file Shall never wrote, so it belongs here rather than in
/// whichever caller happens to reach it.
#[tokio::test]
async fn remove_shim_never_deletes_a_file_shall_did_not_deploy() {
    let tmp = tempdir().unwrap();
    let bin = tmp.path().join("bin");
    let mgr = ShimManager::with_bin_dir(bin.clone()).await.unwrap();

    let victim = bin.join("jq");
    tokio::fs::write(&victim, b"#!/bin/sh\necho the user's own jq\n")
        .await
        .unwrap();

    mgr.remove_shim(
        "jq",
        crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
    )
    .await
    .unwrap();

    assert!(
        victim.exists(),
        "sync deleted a file Shall never created: {:?}",
        victim
    );
}
