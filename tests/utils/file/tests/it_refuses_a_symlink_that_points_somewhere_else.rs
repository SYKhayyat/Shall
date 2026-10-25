use super::*;

#[cfg(unix)]
#[tokio::test]
async fn it_refuses_a_symlink_that_points_somewhere_else() {
    // Another tool's symlink is not ours to replace, even though it is a symlink.
    let (_d, src, bin) = fixture().await;
    let elsewhere = bin.join("some-other-tool");
    tokio::fs::write(&elsewhere, b"x").await.unwrap();
    let dest = bin.join("fd");
    tokio::fs::symlink(&elsewhere, &dest).await.unwrap();

    assert!(deploy_executable(&src, &dest, src.parent().unwrap(), None)
        .await
        .is_err());
}
