use super::*;

#[tokio::test]
async fn it_replaces_the_path_this_backend_recorded_last_time() {
    // The upgrade case: same declaration, new version. A copy carries no pointer home,
    // so the recorded path is what identifies it as ours.
    let (_d, src, bin) = fixture().await;
    let dest = bin.join("fd");
    tokio::fs::write(&dest, b"an older deploy").await.unwrap();

    let recorded = dest.to_string_lossy().to_string();
    deploy_executable(&src, &dest, src.parent().unwrap(), Some(&recorded))
        .await
        .unwrap();
    assert_ne!(
        tokio::fs::read(&dest).await.unwrap_or_default(),
        b"an older deploy".to_vec()
    );
}
