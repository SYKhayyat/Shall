use super::*;

/// A recorded installer Shall has no removal argv for is an error, not a silent skip. The
/// same shape as the failing-manager case, reached without running anything: `remove_argv`
/// refuses first.
#[tokio::test]
async fn an_installer_with_no_known_removal_is_reported_not_skipped() {
    let (web, _mock, _tmp) = backend("odd");
    let url = "https://example.invalid/fd.pkg";
    record(&web.core, url, handed_to("brew", "fd", url)).await;

    let err = web
        .remove(&[url.to_string()], false, reaped().await)
        .await
        .expect_err("an unknown installer must not read as a removal");
    assert!(err.to_string().contains("still on disk"), "{err}");
    assert!(web.core.load_state().await.unwrap().contains_key(url));
}
