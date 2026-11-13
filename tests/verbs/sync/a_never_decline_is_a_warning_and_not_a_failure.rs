use super::*;
use crate::mock_providers::TestKernel;

async fn sync(kernel: &TestKernel) -> Result<()> {
    handle_sync(
        &kernel.app,
        SyncMode {
            locked: false,
            upgrade: false,
        },
        Output::Human,
    )
    .await
}

/// **`#161`, and it is an exit-code contract, not a message.**
///
/// A `[declare] install = "never"` machine whose files and whose state disagree is in exactly the
/// state its owner described: the declaration is in the file and the package is deliberately not
/// on the machine. `sync` used to answer that with exit 1 on every run — through the same
/// non-convergence path a broken manager takes — so a nightly job went red over a machine working
/// as configured, forever. Here it succeeds and says what it did not do.
#[tokio::test]
async fn a_never_decline_is_a_warning_and_not_a_failure() {
    let kernel = declaring(&["dos2unix"], shall::config::config::BareInstall::Never).await;

    sync(&kernel)
        .await
        .expect("a `never` machine did what it was told, so the run must not report a failure");

    let calls = kernel.mock_executor.get_calls().await;
    assert!(
        !calls.iter().any(|c| c.contains("install")),
        "`never` reached the machine with the package it was told not to install: {calls:?}"
    );
}

/// **The control that the decline is not simply an empty plan.** The same fixture, the same
/// declaration, one setting apart: `always` installs it. Without this, the test above passes on a
/// machine that never planned anything.
#[tokio::test]
async fn the_same_declaration_installs_when_the_setting_does_not_forbid_it() {
    let kernel = declaring(&["dos2unix"], shall::config::config::BareInstall::Always).await;

    sync(&kernel)
        .await
        .expect("an `always` machine installs what its files select");

    let calls = kernel.mock_executor.get_calls().await;
    assert!(
        calls.iter().any(|c| c.contains("dos2unix")),
        "the control never reached the manager, so the test above proves nothing: {calls:?}"
    );
}

/// **And the other half of the contract: a real shortfall still fails.** Only *policy* declines
/// are subtracted — a declaration no manager on this machine can take is still work that was
/// asked for and did not happen, and `Q-C` keeps it a failure.
#[tokio::test]
async fn a_declaration_no_manager_can_take_still_fails() {
    let kernel = declaring(&["brew:htop"], shall::config::config::BareInstall::Always).await;
    // A manager that is here and cannot be asked: the third answer, not a policy decline.
    kernel.mock_executor.set_response(
        "brew info --json=v1 -- htop",
        Err(shall::core::Error::command_failed("brew is wedged")),
    );

    let error = sync(&kernel)
        .await
        .expect_err("a declaration no manager can take is not a converged machine");
    assert!(
        error.to_string().contains("not converged"),
        "the failure must say the machine did not converge: {error}"
    );
}
