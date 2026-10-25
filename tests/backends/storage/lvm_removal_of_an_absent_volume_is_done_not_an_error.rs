use super::*;

#[tokio::test]
async fn lvm_removal_of_an_absent_volume_is_done_not_an_error() {
    let (mock, exec) = storage_layer(&[(
        "lvs --noheadings --units b --nosuffix -o vg_name,lv_name vg0/data",
        out(""),
    )]);
    let l = LvmInstallable {
        core: Arc::new(LvmBackendCore {
            executor: exec,
            name: "lvm".to_string(),
        }),
    };
    l.remove(&["vg0/data".to_string()], false, removal_token().await)
        .await
        .expect("already absent is convergence");
    assert!(
        !mock
            .get_calls()
            .await
            .iter()
            .any(|c| c.contains("lvremove")),
        "an absent volume must not be lvremoved: {:?}",
        mock.get_calls().await
    );
}
