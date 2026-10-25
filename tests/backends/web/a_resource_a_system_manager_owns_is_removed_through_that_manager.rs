use super::*;

/// D5: a `.deb` Shall handed to `dpkg` is removed **through `dpkg`**, by the package name
/// read out of the file at install time — not by the URL, and not by deleting a tree Shall
/// does not own.
#[tokio::test]
async fn a_resource_a_system_manager_owns_is_removed_through_that_manager() {
    let (web, mock, _tmp) = backend("deb");
    let url = "https://example.invalid/fd_10.2.0_amd64.deb";
    record(&web.core, url, handed_to("dpkg", "fd", url)).await;

    // Registered as the product will really launch it. A system manager's removal asks
    // for privilege, so on a Linux runner it arrives as `sudo dpkg -r fd` and on Windows
    // as `dpkg -r fd` — three environments, two answers, and `escalates` is the one place
    // that knows which. Hard-coded here, it passed on Windows and had never once run on
    // Linux, because the build matrix was producing one target out of four.
    mock.set_response(&removal_argv("dpkg -r fd"), Ok(DryRunOutput::new().into()));
    // **No reset, and no reset is needed.** The credential answer belongs to the run, and
    // this test's executor is its own run: no other test in this binary can have asked the
    // question already, so the probe this asserts about is the probe that runs. That was
    // not true while the answer was a process-wide flag — then another escalating test
    // answered first, the probe never happened, and this test failed on nothing but
    // libtest's thread order (`#121`).
    web.remove(&[url.to_string()], false, reaped().await)
        .await
        .expect("the removal succeeds");

    // **The credential probe is answered by the mock now, not by the machine running the
    // test.** It used to spawn a real `sudo -n -v`, so this test passed only on a host with
    // passwordless sudo and reported its own stub as unmatched everywhere else
    // (`PLAN.md` #88). Asserted in both directions: the probe went *through* the seam, and the
    // escalated command still carried the argv the product builds.
    let calls = mock.get_calls().await;
    assert!(
        calls.iter().any(|c| c == &removal_argv("dpkg -r fd")),
        "the escalated removal argv never reached the mock: {calls:#?}"
    );
    // **Asked of `escalates`, not assumed, because the answer is a fact about the host.**
    // The probe exists on a platform that escalates with `sudo`; on Windows there is no
    // `sudo` and `escalates` says so, so asserting the probe ran was asserting a POSIX
    // behaviour on a host that has none. Both directions are asserted, so "the probe is
    // gone" and "this host never raises one" cannot be mistaken for each other.
    if CommandExecutor::escalates(true) {
        assert!(
            calls.iter().any(|c| c == "sudo -n -v"),
            "the credential probe went around the layer again, so this test depends on the \
             host having passwordless sudo: {calls:#?}"
        );
    } else {
        assert!(
            !calls.iter().any(|c| c.starts_with("sudo ")),
            "this host does not escalate, and the removal invented an escalation anyway: \
             {calls:#?}"
        );
    }

    let calls = mock.get_calls().await;
    assert!(
        calls.iter().any(|c| c.contains("dpkg -r fd")),
        "the removal did not go through dpkg: {calls:?}"
    );
    assert!(
        !calls.iter().any(|c| c.contains("fd_10.2.0_amd64.deb")),
        "removal named the file rather than the package dpkg lists it under: {calls:?}"
    );
    assert!(
        !web.core.load_state().await.unwrap().contains_key(url),
        "the record survived a successful removal"
    );
}
