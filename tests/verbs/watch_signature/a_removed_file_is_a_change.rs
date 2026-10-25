use super::*;

/// A removed file is a change, because deleting the last profile is a change to what Shall
/// does — and a diff that only reported new and modified bytes calls that a non-event.
#[tokio::test]
async fn a_removed_file_is_a_change() {
    let dir = repo().await;
    std::fs::write(dir.path().join("profiles/Work"), "use dev\n").expect("a profile");
    let before = signature(dir.path()).await;
    std::fs::remove_file(dir.path().join("profiles/Work")).expect("the removal");
    assert_eq!(
        changed(&before, &signature(dir.path()).await),
        vec!["profiles/Work".to_string()]
    );
}
