use super::*;

/// The preview is the enforcer's question, asked without spending anything â€” including the
/// total, which is what a two-list preview with a hand-written `also_removing` could never
/// get right.
#[tokio::test]
async fn the_preview_sees_the_total_the_enforcer_will() {
    let reg = Arc::new(BackendRegistry::new());
    let mut cfg = config_with(0);
    cfg.guard.max_total_changes = 5;
    let refusals = preview_refusals(
        &cfg,
        &reg,
        2,
        &pairs(&["a", "b"]),
        &extras(&["link:/a", "link:/b"]),
        GuardScope::Apply,
    )
    .await;
    assert!(
        refusals
            .iter()
            .any(|r| r.contains("makes 6 changes in total")),
        "two installs, two packages and two teardowns is six: {refusals:?}"
    );
}
