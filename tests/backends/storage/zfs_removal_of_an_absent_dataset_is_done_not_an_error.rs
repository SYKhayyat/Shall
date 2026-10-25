use super::*;

#[tokio::test]
async fn zfs_removal_of_an_absent_dataset_is_done_not_an_error() {
    let (mock, exec) = storage_layer(&[("zfs list -H -o name tank/data", out(""))]);
    let z = ZfsInstallable {
        core: Arc::new(ZfsBackendCore {
            executor: exec,
            name: "zfs".to_string(),
        }),
    };
    z.remove(&["tank/data".to_string()], false, removal_token().await)
        .await
        .expect("already absent is convergence, not a failure");
    assert!(
        !mock.get_calls().await.iter().any(|c| c.contains("destroy")),
        "an absent dataset must not be destroyed: {:?}",
        mock.get_calls().await
    );
}
