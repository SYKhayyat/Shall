use super::*;

/// `@download_only` never linked anything, so `symlink_path` is empty — and an empty path is
/// not a path to delete. The branch exists precisely because treating `""` as a path would
/// make every download-only removal fail.
#[tokio::test]
async fn a_download_only_appimage_has_no_path_entry_to_drop() {
    let (app, tmp) = backend("dl");
    let local = tmp.path().join("dl").join("tool.AppImage");
    touch(&local).await;
    record(
        &app.core,
        "tool",
        AppImageState {
            url: "https://example.invalid/tool.AppImage".into(),
            local_path: local.to_string_lossy().into(),
            symlink_path: String::new(),
        },
    )
    .await;

    app.remove(&["tool".to_string()], false, reaped().await)
        .await
        .expect("a download-only AppImage removes without a link");
    assert!(!local.exists());
    assert!(app.core.load_state().await.unwrap().is_empty());
}
