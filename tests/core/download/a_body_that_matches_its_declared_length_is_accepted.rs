use super::*;

/// A declared length that is met is not a failure — the check must not refuse the ordinary
/// case it was added to bound.
#[tokio::test]
async fn a_body_that_matches_its_declared_length_is_accepted() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dest = dir.path().join("artifact");
    let staged = stage_capped_to(declares(10, "0123456789"), &dest, "an artifact", Some(1024))
        .await
        .expect("a complete body");
    assert_eq!(staged.bytes(), 10);
    staged.commit().await.expect("committed");
    assert_eq!(std::fs::read(&dest).expect("written"), b"0123456789");
}
