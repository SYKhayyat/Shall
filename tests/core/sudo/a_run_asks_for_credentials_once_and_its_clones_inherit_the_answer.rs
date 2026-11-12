use super::*;
use shall::core::executor::DryRunOutput;

/// A run asks the machine for credentials once, and every handle it holds inherits the answer.
///
/// **The saving is the reason the state exists.** Asking again costs the whole password bound
/// again — per escalated command, on the verbs that deliberately keep going after one backend
/// fails — and that is how a 120-second bound produced a 900-second wedge. So the answer is
/// remembered, and a clone of the executor is another handle on one run, not a second run: it
/// must not ask again.
#[tokio::test]
async fn a_run_asks_for_credentials_once_and_its_clones_inherit_the_answer() {
    if !this_host_escalates() {
        return;
    }
    let (executor, mock) = wired();

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response("sudo -n -v", Ok(DryRunOutput::default().into()));
    mock.set_response(
        "sudo -n apt install -y curl",
        Ok(DryRunOutput::default().into()),
    );
    let _ = executor.run("apt", &["install", "-y", "curl"], true).await;
    let calls = mock.get_calls().await;
    assert_eq!(
        probes(&mock).await,
        1,
        "the run did not ask the machine exactly once: {calls:#?}"
    );

    let clone = executor.clone();
    let _ = clone.run("apt", &["install", "-y", "curl"], true).await;
    let calls = mock.get_calls().await;
    assert_eq!(
        probes(&mock).await,
        1,
        "a second handle on the same run asked the machine again: {calls:#?}"
    );
}
