use super::*;

/// **A `{url}` that lands inside an `sh -c` template cannot be allowed to break out of its
/// single quotes.** The pacman and apk rows interpolate the URL into a root shell string;
/// one `'` in it ends the quoted region and everything after is a new command as root.
/// `reject_shell_meta` refuses exactly this — pinned here so the guard cannot quietly
/// narrow while the templates still exist.
#[tokio::test]
async fn a_url_that_could_break_the_shell_template_is_refused_before_anything_runs() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let mgr = apk_repo(mock.clone(), vfs);
    for hostile in [
        "http://mirror.example.invalid/' ; rm -rf / #",
        "http://m.example.invalid/$(curl evil)",
        "http://m.example.invalid/x`reboot`y",
    ] {
        let err = mgr
            .add_repo("mine", hostile, false)
            .await
            .expect_err("a quote in the URL would end the sh -c string");
        assert!(err.to_string().contains("Unsafe"), "{hostile}: {err}");
    }
    assert!(
        mock.get_calls().await.is_empty(),
        "nothing ran for any of them: {:?}",
        mock.get_calls().await
    );
}
