use super::*;

#[tokio::test]
async fn a_manager_that_said_no_lets_the_pick_be_recorded() {
    let (backend, recorded) = settle(&bare_jq(), registry(DryRunOutput::new().into(), found()))
        .await
        .unwrap();
    assert_eq!(backend, "second");
    assert_eq!(recorded.as_deref(), Some("second"));
}
