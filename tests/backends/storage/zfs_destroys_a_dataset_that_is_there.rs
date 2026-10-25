use super::*;

#[tokio::test]
async fn zfs_destroys_a_dataset_that_is_there() {
    let (mock, exec) = storage_layer(&[("zfs list -H -o name tank/data", out("tank/data\n"))]);
    let z = ZfsInstallable {
        core: Arc::new(ZfsBackendCore {
            executor: exec,
            name: "zfs".to_string(),
        }),
    };
    z.remove(&["tank/data".to_string()], false, removal_token().await)
        .await
        .expect("present dataset is removed");
    assert!(
        mock.get_calls()
            .await
            .iter()
            .any(|c| c.contains("zfs destroy -r tank/data")),
        "the present dataset was never destroyed: {:?}",
        mock.get_calls().await
    );
}
