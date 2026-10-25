use super::*;

#[tokio::test]
async fn free_path_steps_past_a_fallback_that_is_also_taken() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();

    // Nothing taken: the plain `.shall` name.
    assert_eq!(
        free_path(d, "package.json").await,
        d.join("package.shall.json")
    );

    // The fallback itself exists, so a second export must not clobber the first one's.
    tokio::fs::write(d.join("package.shall.json"), "first")
        .await
        .unwrap();
    assert_eq!(
        free_path(d, "package.json").await,
        d.join("package.shall.json.2")
    );
}
