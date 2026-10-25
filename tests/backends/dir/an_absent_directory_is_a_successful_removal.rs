use super::*;

#[tokio::test]
async fn an_absent_directory_is_a_successful_removal() {
    let root = tempdir().unwrap();
    installer()
        .remove(
            &[root.path().join("never-existed").display().to_string()],
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Sync).await,
        )
        .await
        .expect("there is nothing left to remove, which is the state the caller wanted");
}
