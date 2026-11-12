use super::*;
use shall::core::executor::DryRunOutput;

/// The `rpm` half of the same rule. Written because `install_argv` deliberately differs
/// between the two (`dpkg -i` vs `rpm -U --replacepkgs`), and a test on only one of them
/// would pin the pair's shape while leaving the divergent one unasked.
#[tokio::test]
async fn the_rpm_twin_removes_by_name_too() {
    let (web, mock, _tmp) = backend("rpm");

    // Stated rather than fallen through (`#162`): what this test asks its manager, and the
    // answer it read from silence before. The assertions here are about the argv.
    mock.set_response("sudo -n -v", Ok(DryRunOutput::default().into()));
    let url = "https://example.invalid/fd-10.2.0.x86_64.rpm";
    record(&web.core, url, handed_to("rpm", "fd", url)).await;

    mock.set_response(&removal_argv("rpm -e fd"), Ok(DryRunOutput::new().into()));
    web.remove(&[url.to_string()], false, reaped().await)
        .await
        .expect("the removal succeeds");

    let calls = mock.get_calls().await;
    assert!(
        calls.iter().any(|c| c.contains("rpm -e fd")),
        "the removal did not go through rpm: {calls:?}"
    );
}
