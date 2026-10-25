use super::*;

#[tokio::test]
async fn a_directory_that_cannot_be_emptied_is_a_failure_and_not_a_removal() {
    let root = tempdir().unwrap();
    let busy = root.path().join("busy");
    std::fs::create_dir(&busy).unwrap();
    std::fs::write(busy.join("the-users-data"), "keep me\n").unwrap();

    let error = installer()
        .remove(
            &[busy.display().to_string()],
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Sync).await,
        )
        .await
        .expect_err("a directory that was left behind is not a successful removal");
    assert!(error.to_string().contains("not empty"), "{error}");
    assert!(
        busy.join("the-users-data").exists(),
        "the contents were destroyed by a teardown that could not remove the directory"
    );
}
