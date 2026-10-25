use super::*;

#[tokio::test]
async fn it_refuses_to_replace_a_file_shall_did_not_deploy() {
    // `~/.local/bin` is shared with the user. Deploying by name alone would make a
    // package called `fd` silently destroy whatever `fd` they already had.
    let (_d, src, bin) = fixture().await;
    let dest = bin.join("fd");
    tokio::fs::write(&dest, b"the user's own fd").await.unwrap();

    let err = deploy_executable(&src, &dest, src.parent().unwrap(), None)
        .await
        .unwrap_err();
    assert!(format!("{}", err).contains("did not create it"), "{}", err);
    // And it is still theirs.
    assert_eq!(
        tokio::fs::read(&dest).await.unwrap(),
        b"the user's own fd".to_vec()
    );
}
