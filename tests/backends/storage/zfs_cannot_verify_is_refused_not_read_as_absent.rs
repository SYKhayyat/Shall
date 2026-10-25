use super::*;

#[tokio::test]
async fn zfs_cannot_verify_is_refused_not_read_as_absent() {
    // The M5 posture at the effector: an ask that fails is not "nothing there", or a
    // listing outage would skip the destroy in silence.
    let (mock, exec) = storage_layer(&[(
        "zfs list -H -o name tank/data",
        Err(Error::Io("zfs exited 2".into())),
    )]);
    let z = ZfsInstallable {
        core: Arc::new(ZfsBackendCore {
            executor: exec,
            name: "zfs".to_string(),
        }),
    };
    let e = z
        .remove(&["tank/data".to_string()], false, removal_token().await)
        .await
        .expect_err("cannot verify must refuse, not skip");
    assert!(!e.to_string().is_empty());
    assert!(
        !mock.get_calls().await.iter().any(|c| c.contains("destroy")),
        "no destroy runs on an unverifiable ask"
    );
}
