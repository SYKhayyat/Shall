use super::*;

/// helm's own failure names `--verify=false`, an argv no declaration can write. The advice
/// a user can act on is the flag on the line.
#[tokio::test]
async fn a_verification_failure_names_the_flag_a_declaration_can_write() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_response(
        "helm plugin install -- https://github.com/databus23/helm-diff",
        Err(shall::core::Error::CommandFailed {
            message: "Error: plugin source does not support verification. Use --verify=false \
                      to skip verification"
                .into(),
            retry: shall::core::Retryability::Permanent,
            // Permanent and the plugin's name is fine — the fixture for the distinction
            // `absent_name` exists to draw.
            absent_name: false,
        }),
    );
    let inst = GenericInstallable {
        core: Arc::new(source_option_core(mock.clone(), vfs)),
    };

    let msg = inst
        .install(
            &[spec_with(
                "diff",
                &[("url", "https://github.com/databus23/helm-diff")],
            )],
            false,
        )
        .await
        .unwrap_err()
        .to_string();
    assert!(
        msg.contains("@unverified"),
        "the failure must name the flag the line can carry: {}",
        msg
    );
}
