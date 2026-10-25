use super::*;

/// Both deployed paths go, and the record with them.    /// Both deployed paths go, and the record with them.
#[tokio::test]
async fn removing_an_appimage_takes_the_file_and_the_path_entry() {
    let (app, tmp) = backend("img");
    let local = tmp.path().join("img").join("fd.AppImage");
    let link = tmp.path().join("bin").join("fd");
    touch(&local).await;
    touch(&link).await;
    record(
        &app.core,
        "fd",
        AppImageState {
            url: "https://example.invalid/fd.AppImage".into(),
            local_path: local.to_string_lossy().into(),
            symlink_path: link.to_string_lossy().into(),
        },
    )
    .await;

    app.remove(&["fd".to_string()], false, reaped().await)
        .await
        .expect("the removal succeeds");

    assert!(!local.exists(), "the AppImage is still on disk");
    assert!(!link.exists(), "the PATH entry survived the removal");
    assert!(!app.core.load_state().await.unwrap().contains_key("fd"));
}
