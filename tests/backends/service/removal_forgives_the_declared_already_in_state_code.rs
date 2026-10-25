use super::*;

#[tokio::test]
async fn removal_forgives_the_declared_already_in_state_code() {
    // 42 is this row's "already stopped": success for a converger, per benign_exits.
    let (_mock, exec) = service_layer(&[
        ("stop-cmd nginx", exit_of(42)),
        ("disable-cmd nginx", exit_of(0)),
    ]);
    core_with(exec)
        .remove(&["nginx".to_string()], false, removal_token().await)
        .await
        .expect("the declared benign code is convergence");
}
