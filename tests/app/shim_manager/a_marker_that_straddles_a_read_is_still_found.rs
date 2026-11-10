use super::*;
use tokio::io::AsyncWriteExt;

/// The marker scan is a streaming search, so a marker that falls across the boundary between
/// two reads must still be found. This file is a shade over one megabyte with the marker
/// placed so it begins ten bytes before the 64 KiB chunk ends.
#[tokio::test]
async fn a_marker_that_straddles_a_read_is_still_found() {
    let tmp = tempdir().unwrap();
    let bin = tmp.path().join("bin");
    tokio::fs::create_dir_all(&bin).await.unwrap();
    let deployed = bin.join(if cfg!(windows) {
        "ripgrep.exe"
    } else {
        "ripgrep"
    });

    let mut bytes = vec![0u8; 1024 * 1024];
    let at = 64 * 1024 - 10;
    bytes[at..at + SHIM_MARKER.len()].copy_from_slice(SHIM_MARKER);
    let mut file = tokio::fs::File::create(&deployed).await.unwrap();
    file.write_all(&bytes).await.unwrap();
    file.flush().await.unwrap();

    let mgr = ShimManager::with_bin_dir(bin.clone()).await.unwrap();
    let shims = mgr.list_shims().await.unwrap();

    assert!(
        shims.iter().any(|s| s.starts_with("ripgrep")),
        "a marker split across two reads was not found, so a real shim of this size would be \
         invisible: {shims:?}"
    );
}
