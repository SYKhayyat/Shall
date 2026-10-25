use super::*;

/// **The same rule for the JSON records, because the same bug lived in three of them.**
///
/// A truncated `web`/`github`/`appimage` state file read as an empty map, and the map was
/// then merged into and written back — so the second run made the loss permanent. Absent
/// is still the correct starting state; unparseable is not.
#[tokio::test]
async fn a_corrupt_json_record_is_an_error_and_a_missing_one_is_empty() {
    let tmp = TempDir::new().unwrap();
    let missing = tmp.path().join("never-written.json");
    let empty: std::collections::HashMap<String, String> =
        load_json_records(&missing).await.unwrap();
    assert!(empty.is_empty(), "a missing record starts empty");

    let corrupt = tmp.path().join("torn.json");
    std::fs::write(&corrupt, "{\"fd\": {\"version\"").unwrap();
    let err = load_json_records::<String>(&corrupt).await.unwrap_err();
    assert!(
        matches!(err, Error::Json(_)),
        "a corrupt record must not read as empty; got {err:?}"
    );
}
