use super::*;

/// Removal batches the same way.
#[tokio::test]
async fn a_batch_of_removals_is_one_emacs() {
    let (core, mock) = wired();
    EmacsInstallable { core }
        .remove(
            &["csv-mode".to_string(), "rainbow-mode".to_string()],
            false,
            crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
        )
        .await
        .unwrap();
    let calls = mock.get_calls().await;
    assert_eq!(calls.len(), 1, "{:?}", calls);
    assert!(calls[0].contains("package-delete"), "{:?}", calls);
}
