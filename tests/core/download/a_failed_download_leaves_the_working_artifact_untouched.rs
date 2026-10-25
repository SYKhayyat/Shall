use super::*;

/// **The artifact that is already there survives a failed download of its replacement.**
///
/// This is the defect: the old code opened the destination and streamed into it, then deleted
/// it on the way out, so a re-download that died at 60% left a machine with no binary *and* no
/// record of why. The error said the transfer failed; the PATH symlink pointed at nothing.
#[tokio::test]
async fn a_failed_download_leaves_the_working_artifact_untouched() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dest = dir.path().join("tool");

    stage_capped_to(undeclared("the good version"), &dest, "a tool", None)
        .await
        .expect("the first download")
        .commit()
        .await
        .expect("committed");
    assert_eq!(
        std::fs::read(&dest).expect("the artifact"),
        b"the good version"
    );

    let err = stage_capped_to(breaks_after("half of a new ver"), &dest, "a tool", None)
        .await
        .expect_err("a dropped connection is not a download");
    assert_eq!(
        std::fs::read(&dest).expect("the artifact is still there"),
        b"the good version",
        "a failed re-download destroyed the working artifact"
    );
    assert!(
        err.to_string().contains("still in place"),
        "a download that failed over a working artifact did not say so: {err}"
    );
}
