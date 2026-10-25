use super::*;

/// A body that was staged and dropped without a commit leaves the destination alone, which is
/// the other half of the contract: the `Drop` is the cleanup, so a caller that verifies a
/// checksum and finds a mismatch has to say so by returning — and cannot leak by forgetting.
#[tokio::test]
async fn a_staged_body_dropped_without_a_commit_takes_nothing_else_with_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dest = dir.path().join("tool");
    stage_capped_to(undeclared("the good version"), &dest, "a tool", None)
        .await
        .expect("the first download")
        .commit()
        .await
        .expect("committed");

    {
        let _refused =
            stage_capped_to(undeclared("bytes that do not match"), &dest, "a tool", None)
                .await
                .expect("staged");
        assert!(
            part_path(&dest).exists(),
            "a staged file must exist while it is staged, or the checksum has nothing to read"
        );
    }

    assert!(
        !part_path(&dest).exists(),
        "a staged file that was never committed survived its owner"
    );
    assert_eq!(
        std::fs::read(&dest).expect("the artifact"),
        b"the good version"
    );
}
