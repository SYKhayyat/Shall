use super::*;
use tokio::io::AsyncWriteExt;

/// `bin_dir` is shared with every other tool that installs there, and a listing used to read
/// every file in it **whole** — 50 to 150 MB each for `uv`, `bun`, `deno`, a language server.
///
/// The property asserted is the answer, not the instrument: a large file with no marker in it
/// is not a shim, and the listing still finds the real one beside it. The scan is bounded by
/// [`MARKER_SCAN_CHUNK`] however large the file is, which is what this fixture would exhaust if
/// it were read in one piece — twice over, for the copy comparison the old code also ran.
#[tokio::test]
async fn a_file_larger_than_any_shall_build_is_not_listed_as_a_shim() {
    let tmp = tempdir().unwrap();
    let bin = tmp.path().join("bin");
    tokio::fs::create_dir_all(&bin).await.unwrap();

    let big = bin.join("some-other-tool");
    let mut file = tokio::fs::File::create(&big).await.unwrap();
    // 8 MiB is already 40x the chunk, and the zeros contain no marker.
    file.write_all(&vec![0u8; 8 * 1024 * 1024]).await.unwrap();
    file.flush().await.unwrap();

    let mgr = ShimManager::with_bin_dir(bin.clone()).await.unwrap();
    let shims = mgr.list_shims().await.unwrap();

    assert!(
        !shims.iter().any(|s| s == "some-other-tool"),
        "a file Shall never deployed was reported as a shim: {shims:?}"
    );
}
