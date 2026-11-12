use super::*;
use shall::core::executor::DryRunOutput;

/// Two runs are two answers.
///
/// **This is the whole of `#121`.** The answer used to be a process-wide flag, so a test that
/// wanted to watch the credential probe was at the mercy of whichever test in the binary had
/// asked first — the web backend's test failed on nothing but libtest's thread order, and took
/// the mutation gate's baseline down with it. The answer belongs to the run, so a second run
/// asks for its own.
#[tokio::test]
async fn a_second_run_asks_for_its_own_credentials() {
    if !this_host_escalates() {
        return;
    }
    let (first, first_mock) = wired();

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    first_mock.set_response("sudo -n -v", Ok(DryRunOutput::default().into()));
    first_mock.set_response(
        "sudo -n apt install -y curl",
        Ok(DryRunOutput::default().into()),
    );
    let (second, second_mock) = wired();
    // The second run asks its own questions, so it gets its own answers — the point of the test
    // is that it asked at all, and both used to be read out of silence (`#162`).
    second_mock.set_response("sudo -n -v", Ok(DryRunOutput::default().into()));
    second_mock.set_response(
        "sudo -n apt install -y curl",
        Ok(DryRunOutput::default().into()),
    );

    let _ = first.run("apt", &["install", "-y", "curl"], true).await;
    let _ = second.run("apt", &["install", "-y", "curl"], true).await;

    let first_calls = first_mock.get_calls().await;
    let second_calls = second_mock.get_calls().await;
    assert_eq!(
        probes(&first_mock).await,
        1,
        "the first run did not ask the machine exactly once: {first_calls:#?}"
    );
    assert_eq!(
        probes(&second_mock).await,
        1,
        "the second run inherited the first run's answer and never asked: {second_calls:#?}"
    );
}
