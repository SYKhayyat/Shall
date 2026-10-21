use super::*;

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
    let (second, second_mock) = wired();

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
