use super::*;

#[cfg(unix)]
#[tokio::test]
async fn it_replaces_a_symlink_that_points_into_its_own_artifacts() {
    let (_d, src, bin) = fixture().await;
    let old = src.parent().unwrap().join("fd-old");
    tokio::fs::write(&old, b"old").await.unwrap();
    let dest = bin.join("fd");
    tokio::fs::symlink(&old, &dest).await.unwrap();

    deploy_executable(&src, &dest, src.parent().unwrap(), None)
        .await
        .unwrap();
    assert_eq!(tokio::fs::read_link(&dest).await.unwrap(), src);
}
