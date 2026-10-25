use super::*;

/// S4: the create path had the same blind spot the remove path used to — it deleted
/// whatever sat at `~/.local/bin/<name>` before deploying. A managed package named `jq`
/// would then clobber the user's own `jq` on the next sync. Deploy must refuse, not
/// destroy, an unmanaged file.
#[tokio::test]
async fn create_shim_refuses_to_clobber_a_file_shall_did_not_deploy() {
    let tmp = tempdir().unwrap();
    let bin = tmp.path().join("bin");
    let mgr = ShimManager::with_bin_dir(bin.clone()).await.unwrap();

    // Windows shims carry `.exe`, so the file in the way is `jq.exe` there. Naming the
    // victim `jq` on Windows tests a path `create_shim` never touches.
    let victim = bin.join(if cfg!(windows) { "jq.exe" } else { "jq" });
    let contents = b"#!/bin/sh\necho the user's own jq\n";
    tokio::fs::write(&victim, contents).await.unwrap();

    let result = mgr.create_shim("jq").await;

    assert!(
        result.is_err(),
        "create_shim must refuse to overwrite a user's file"
    );
    // And it must not have touched the file on its way to refusing.
    let after = tokio::fs::read(&victim).await.unwrap();
    assert_eq!(
        after, contents,
        "the user's file was modified despite the refusal"
    );
}
