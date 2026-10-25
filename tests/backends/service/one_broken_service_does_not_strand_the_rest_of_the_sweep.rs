use super::*;

#[tokio::test]
async fn one_broken_service_does_not_strand_the_rest_of_the_sweep() {
    let (mock, exec) = service_layer(&[
        ("stop-cmd a", exit_of(1)),
        ("disable-cmd a", exit_of(1)),
        ("stop-cmd b", exit_of(0)),
        ("disable-cmd b", exit_of(0)),
    ]);
    let e = core_with(exec)
        .remove(
            &["a".to_string(), "b".to_string()],
            false,
            removal_token().await,
        )
        .await
        .expect_err("`a` failed for real");
    assert!(
        e.to_string().contains('a'),
        "the failure names what broke: {e}"
    );
    let calls = mock.get_calls().await;
    for want in ["stop-cmd a", "disable-cmd a", "stop-cmd b", "disable-cmd b"] {
        assert!(
            calls.iter().any(|c| c.contains(want)),
            "`{want}` was never attempted — the sweep stranded it behind a's failure"
        );
    }
}
