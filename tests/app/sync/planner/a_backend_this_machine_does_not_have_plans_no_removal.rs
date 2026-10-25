use super::*;

/// The other side of the line II.7c drew, and the reason the fixture above had to change:
/// a manager that is **not on this machine** removes nothing, because there is nothing
/// installed through it to remove. Before this, the plan scheduled the removal anyway and
/// the transaction failed it with `BackendNotFound` — a command that could never run.
#[tokio::test]
async fn a_backend_this_machine_does_not_have_plans_no_removal() {
    let changes = absent_removals(Arc::new(BackendRegistry::new()), "libreoffice").await;
    assert_eq!(
        changes, 0,
        "a manager that is not here cannot be asked to remove anything"
    );
}
