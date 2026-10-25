use super::*;

/// The twin of `web.rs`'s. A path the OS refuses leaves the AppImage installed, so the record
/// must go back: dropping it would make the resource drift **no sync can see**, because Shall
/// would have forgotten the only thing that knows it is there.
///
/// The undeletable path is a NUL byte, which every platform's path API refuses with
/// `InvalidInput` rather than `NotFound` — synthetic, deliberately, because the invariant
/// under test is what happens *after* a removal fails and not which removals fail.
#[tokio::test]
async fn a_failed_removal_keeps_the_record_rather_than_forgetting_the_appimage() {
    let (app, _tmp) = backend("stuck");
    record(
        &app.core,
        "wedged",
        AppImageState {
            url: "https://example.invalid/wedged.AppImage".into(),
            local_path: "no\0such".into(),
            symlink_path: String::new(),
        },
    )
    .await;

    let err = app
        .remove(&["wedged".to_string()], false, reaped().await)
        .await
        .expect_err("a path that cannot be removed must not read as a removal");
    assert!(
        err.to_string().contains("still installed"),
        "the error does not say the AppImage is still there: {err}"
    );
    assert!(
        app.core.load_state().await.unwrap().contains_key("wedged"),
        "the record was dropped for an AppImage that is still installed"
    );
}
