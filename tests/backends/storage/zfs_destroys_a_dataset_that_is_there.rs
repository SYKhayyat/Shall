use super::*;
use shall::core::executor::DryRunOutput;

#[tokio::test]
async fn zfs_destroys_a_dataset_that_is_there() {
    let (mock, exec) = storage_layer(&[("zfs list -H -o name tank/data", out("tank/data\n"))]);

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "zfs destroy -r tank/data",
        Ok(DryRunOutput::default().into()),
    );

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "zfs destroy -r tank/data",
        Ok(DryRunOutput::default().into()),
    );
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
