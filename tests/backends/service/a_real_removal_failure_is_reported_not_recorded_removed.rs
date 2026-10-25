use super::*;

#[tokio::test]
async fn a_real_removal_failure_is_reported_not_recorded_removed() {
    let (_mock, exec) = service_layer(&[
        ("stop-cmd nginx", exit_of(42)),
        ("disable-cmd nginx", exit_of(1)),
    ]);
    let e = core_with(exec)
        .remove(&["nginx".to_string()], false, removal_token().await)
        .await
        .expect_err("a masked-broken unit must not read as removed");
    assert!(e.to_string().contains("nginx"), "{e}");
}
