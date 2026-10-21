use super::*;

/// **The refusal names the package and the rule that protected it.** The code says so itself, in
/// the doc comment above `enforce_own_work`:
///
/// > The refusal names the package, so "we could not fully unwind" is answerable.
///
/// A refusal that does not say which package is a refusal the user has to guess about, and the
/// two `refusal_for_own_work` match arms are the only place that name is written. Deleting
/// either arm leaves a refusal that still refuses and no longer says what for — which is the
/// second mutation the gate reported as surviving.
#[tokio::test]
async fn the_refusal_names_the_package_it_refused_to_undo() {
    let reg = registry_reporting("apt", &[]);
    let cfg = config_with(0);
    let err = enforce_own_work(&cfg, &reg, &pairs(&["python3"]))
        .await
        .expect_err("an undo that would remove a protected package must be refused");
    let msg = err.to_string();
    assert!(
        msg.contains("apt:python3"),
        "the refusal names the package by backend and name: {msg}"
    );
    assert!(
        msg.contains("protected by config rule `python3`"),
        "the refusal says which rule protected it, so the user knows what to change: {msg}"
    );
}
