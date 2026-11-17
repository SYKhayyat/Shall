use super::*;
use dashmap::DashMap;
use shall::core::executor::{DryRunOutput, MockExecutor};
use std::sync::Arc;

/// **The four words Task Scheduler answers with, and the sentence a person needs instead.**
///
/// `remove_task` was `let _ = executor.run("schtasks", ["/Delete", …])` and then, when the task
/// was still there, `the scheduled task … still exists after removal`. So the one party that knew
/// *why* threw the answer away and reported the symptom — and the symptom is indistinguishable
/// from a scheduler bug. Measured on this repository's Windows host, where the token is filtered
/// and `Access is denied.` is the whole truth (#173):
///
/// ```text
/// $ schtasks /Delete /TN Shall_nightly /F
/// ERROR: Access is denied.
/// ```
///
/// `add_task` had already been taught to say what to do about it, because `/Create` answers with
/// the same four words. The defect is that the diagnosis was written at one call site instead of
/// being a reading of the error that both call sites ask — so the second one kept the symptom.
///
/// The test drives the real `remove_task` against a mock: `/Delete` refuses for elevation and
/// `/Query` shows the task is still registered, which is the measured shape. It is not
/// `#[cfg(windows)]` because the mock is not Windows and the observation — *the reason the OS
/// gave is the one that reaches the user* — is a property of this file on every platform. What is
/// Windows-only is the string `schtasks` prints, and that is quoted from the measurement above.
#[tokio::test]
async fn a_refused_removal_says_what_to_do_not_what_is_wrong() {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));

    // The task is there, before and after — `/Query` succeeding is what `is_task_active` reads as
    // "still registered", and it is why the removal reports anything at all.
    mock.set_response(
        &CommandExecutor::as_launched("schtasks", &["/Query", "/TN", "Shall_nightly"], false),
        Ok(DryRunOutput::new().into()),
    );
    let refused = Error::Io("ERROR: Access is denied.".into());
    mock.set_response(
        &CommandExecutor::as_launched("schtasks", &["/Delete", "/TN", "Shall_nightly", "/F"], true),
        Err(refused),
    );

    let executor = CommandExecutor::with_layer(false, false, mock, vfs, Default::default());
    let err = WindowsTaskProvisioner
        .remove_task(&executor, "nightly", &here())
        .await
        .expect_err("the task is still registered, so the removal did not happen");

    // **`Permission`, not `Io`.** The run declined because of where it was run from, which is the
    // class every other elevation refusal in this program uses — and the one the exit codes in
    // `src/core/exit.rs` are defined in terms of. Reported as `Io`, a script retrying on failure
    // retries it for ever.
    assert!(
        matches!(err, Error::Permission(_)),
        "a removal refused for lack of elevation is not an I/O fault: {:?}",
        err
    );
    let said = err.to_string();
    for must in ["elevated shell", "Run as administrator", "nightly"] {
        assert!(
            said.contains(must),
            "the refusal does not say `{}`, so it names neither the cause nor the cure: {}",
            must,
            said
        );
    }
    assert!(
        !said.contains("still exists after removal"),
        "the symptom was reported instead of the cause, which is what #173 was: {}",
        said
    );
}

/// The other half, and the reason this is a *reading* of the error rather than a rewrite of the
/// message: an answer that is **not** about elevation must be forwarded, not replaced. A blanket
/// "needs an elevated shell" over every failed `/Delete` would be a new falsehood in the same
/// place as the old one — a task named wrongly, or a store whose permissions are broken, would
/// send the user looking for an administrator prompt that is not the problem.
#[tokio::test]
async fn a_removal_that_failed_for_another_reason_quotes_that_reason() {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_response(
        &CommandExecutor::as_launched("schtasks", &["/Query", "/TN", "Shall_nightly"], false),
        Ok(DryRunOutput::new().into()),
    );
    mock.set_response(
        &CommandExecutor::as_launched("schtasks", &["/Delete", "/TN", "Shall_nightly", "/F"], true),
        Err(Error::Io("ERROR: The parameter is incorrect.".into())),
    );

    let executor = CommandExecutor::with_layer(false, false, mock, vfs, Default::default());
    let err = WindowsTaskProvisioner
        .remove_task(&executor, "nightly", &here())
        .await
        .expect_err("the task is still registered, so the removal did not happen");

    let said = err.to_string();
    assert!(
        said.contains("The parameter is incorrect"),
        "the reason the scheduler gave was discarded, which is the shape of the bug this fixes: {}",
        said
    );
    assert!(
        !said.contains("elevated"),
        "a failure that is not about elevation was blamed on elevation: {}",
        said
    );
}
