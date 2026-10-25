use super::*;

/// **The name `list` returns is the name `install` was handed.**
///
/// `fetch_installed` reported `url.split('/').next_back()` — the basename — while `install`
/// keys the state by the whole URL, which is also what the declaration says. So `info(url)`
/// never matched, every declared AppImage read as absent, and `sync` re-downloaded all of
/// them on every run for ever. `btrfs:` had the same bug, diagnosed in those words and fixed
/// on 2026-07-30; `web:`, this backend's twin, never had it.
#[tokio::test]
async fn an_appimage_is_listed_by_the_url_install_was_given() {
    let (app, tmp) = backend("img");
    let url = "https://example.invalid/dl/v2.1/fd-v2.1-x86_64.AppImage";
    record(
        &app.core,
        url,
        AppImageState {
            url: url.into(),
            local_path: tmp
                .path()
                .join("img")
                .join("fd-v2.1-x86_64.AppImage")
                .to_string_lossy()
                .into(),
            symlink_path: String::new(),
        },
    )
    .await;
    let q = AppImageQueryable {
        core: app.core.clone(),
    };
    let names: Vec<String> = q
        .list_installed()
        .await
        .expect("the listing")
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert_eq!(
        names,
        vec![url.to_string()],
        "`install` was handed the URL, so `list` has to say the same string"
    );
    // The consequence, asserted where it bit: the planner asks `info` with the declared name.
    assert!(
        q.info(url).await.expect("the query").is_some(),
        "a declared AppImage read as absent — this is the re-download-for-ever bug"
    );
    assert!(
        q.info("fd-v2.1-x86_64.AppImage")
            .await
            .expect("the query")
            .is_none(),
        "the basename is not a name anything declares, so it must not answer either"
    );
}
