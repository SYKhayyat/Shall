use super::*;

#[tokio::test]
async fn a_record_with_nothing_deployed_removes_cleanly() {
    let errors = tear_down(&Deployed::default(), &exec(), true, &[]).await;
    assert!(errors.is_empty(), "{errors:?}");
}
