use super::*;

/// `0` means no bound, and it has to mean that all the way down — a `Some(0)` would refuse
/// every download instead of allowing every one.
#[tokio::test]
async fn no_ceiling_writes_anything() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dest = dir.path().join("artifact");
    let staged = stage_capped_to(undeclared("0123456789"), &dest, "an artifact", None)
        .await
        .expect("no ceiling refuses nothing");
    assert_eq!(staged.bytes(), 10);
    staged.commit().await.expect("the commit");
}
