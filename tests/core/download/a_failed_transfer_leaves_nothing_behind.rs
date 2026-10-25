use super::*;

/// **Every error exit takes the staged file with it, and the destination is never touched.**
///
/// The cap refusal cleaned up and said why; the dropped connection, the failed write and the
/// failed flush did not. What is different now is *which* file the partial lives in: it is a
/// sibling, and the `Drop` on `Staged` removes it on every exit, so there is no arm left that
/// can forget. A staged file left behind by a `SIGKILL` is the one case `Drop` cannot cover,
/// which is why the next attempt removes it on entry.
#[tokio::test]
async fn a_failed_transfer_leaves_nothing_behind() {
    let dir = tempfile::tempdir().expect("tempdir");

    let dropped = dir.path().join("dropped");
    stage_capped_to(breaks_after("012345"), &dropped, "an artifact", None)
        .await
        .expect_err("a connection that goes away is not a download");
    assert!(
        !dropped.exists(),
        "a partial file survived a dropped connection"
    );
    assert!(
        !part_path(&dropped).exists(),
        "the staged file survived a dropped connection"
    );

    let short = dir.path().join("short");
    let err = stage_capped_to(declares(10, "012345"), &short, "an artifact", None)
        .await
        .expect_err("six bytes of a declared ten is not the file");
    assert!(err.to_string().contains("cut short"), "{err}");
    assert!(!short.exists(), "a truncated file survived a short body");
    assert!(
        !part_path(&short).exists(),
        "the staged file survived a short body"
    );
}
