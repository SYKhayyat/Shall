use super::*;

/// **One protected package refuses the whole unwind, and the machine keeps everything.** The
/// alternative is a machine holding one package the manifest asks for and not the nine it asked
/// to undo — a state neither the user nor the manifest describes, and one nobody would believe a
/// rollback had produced.
///
/// This is the assertion the nightly's mutation gate wanted and could not get: deleting the `!`
/// in `enforce_own_work`'s refusal inverts the door, so the protected package is removed and this
/// test is the thing that notices.
#[tokio::test]
async fn a_protected_package_stops_the_whole_unwind() {
    let reg = registry_reporting("apt", &[]);
    let cfg = config_with(0);
    // "python3" is on `config_with`'s protected list; "htop" is not, and is not on any default.
    let err = enforce_own_work(&cfg, &reg, &pairs(&["htop", "python3"]))
        .await
        .expect_err("an undo that would remove a protected package must be refused");
    let msg = err.to_string();
    assert!(
        msg.contains("python3"),
        "the refusal has to name the package it refused to undo: {msg}"
    );
    assert!(
        msg.contains("2 package(s) this run installed, left in place"),
        "all or nothing means the refusal says the whole set is staying: {msg}"
    );
}
