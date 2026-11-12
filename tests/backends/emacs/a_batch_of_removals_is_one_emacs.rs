use super::*;
use shall::core::executor::DryRunOutput;

/// Removal batches the same way.
#[tokio::test]
async fn a_batch_of_removals_is_one_emacs() {
    let (core, mock) = wired();

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "emacs --batch --eval (progn (require 'package) (package-initialize) (dolist (n '(csv-mode rainbow-mode)) (let ((p (cadr (assoc n package-alist)))) (if p (package-delete p)))))",
        Ok(DryRunOutput::default().into()),
    );

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response(
        "emacs --batch --eval (progn (require 'package) (package-initialize) (dolist (n '(csv-mode rainbow-mode)) (let ((p (cadr (assoc n package-alist)))) (if p (package-delete p)))))",
        Ok(DryRunOutput::default().into()),
    );
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
