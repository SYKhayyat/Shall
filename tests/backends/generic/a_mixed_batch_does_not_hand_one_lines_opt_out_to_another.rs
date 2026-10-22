use super::*;

/// One package's opt-out is not another's. A batch where the specs disagree has to become
/// two commands, or the flag silently covers a line that never asked for it — which is the
/// global-switch failure `@unverified` is per-line to avoid.
#[tokio::test]
async fn a_mixed_batch_does_not_hand_one_lines_opt_out_to_another() {
    if !this_hosts_helm_takes_the_opt_out() {
        eprintln!("this host's helm rejects --verify; the premise of this test is false here");
        return;
    }
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let inst = GenericInstallable {
        core: Arc::new(source_option_core(mock.clone(), vfs)),
    };

    inst.install(
        &[
            spec_with("diff", &[("url", "https://example.com/helm-diff")]),
            spec_with(
                "secrets",
                &[
                    ("url", "https://example.com/helm-secrets"),
                    ("unverified", "true"),
                ],
            ),
        ],
        false,
    )
    .await
    .unwrap();

    let calls = mock.get_calls().await;
    let with_flag: Vec<&String> = calls
        .iter()
        .filter(|c| c.contains("--verify=false"))
        .collect();
    assert_eq!(
        with_flag.len(),
        1,
        "exactly one of the two asked to skip verification: {:?}",
        calls
    );
    assert!(
        with_flag[0].contains("helm-secrets") && !with_flag[0].contains("helm-diff"),
        "the flag went out with the wrong package: {:?}",
        calls
    );
}
