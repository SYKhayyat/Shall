use super::*;

/// U19's rule, on the reading side: a `@scope=system` line against a store with no
/// machine-wide commands is unanswerable, not answered from the per-user key. Reading one
/// scope and writing the other is the bug `@scope=` was carried into the ledger to fix, and
/// it would arrive here as a confident `Some`.
#[tokio::test]
async fn system_scope_on_a_user_only_store_is_unanswerable() {
    let core = store_reading("prefer-dark");
    assert_eq!(
        core.holds("org.gnome.x/k", "prefer-dark", Some("system"), None)
            .await,
        None
    );
    // ...and the same store answers the scope it does have.
    assert_eq!(
        core.holds("org.gnome.x/k", "prefer-dark", Some("user"), None)
            .await,
        Some(true)
    );
}
