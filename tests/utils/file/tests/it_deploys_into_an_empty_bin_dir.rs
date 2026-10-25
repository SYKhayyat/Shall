use super::*;

#[tokio::test]
async fn it_deploys_into_an_empty_bin_dir() {
    let (_d, src, bin) = fixture().await;
    let dest = bin.join("fd");
    deploy_executable(&src, &dest, src.parent().unwrap(), None)
        .await
        .unwrap();
    assert!(tokio::fs::symlink_metadata(&dest).await.is_ok());
}
