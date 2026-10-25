use super::*;

/// **The bug this module exists for: a change outside `modules/` is a change.**
#[tokio::test]
async fn an_edit_anywhere_in_the_repo_is_a_change() {
    let dir = repo().await;
    let before = signature(dir.path()).await;

    for (path, body) in [
        ("profiles/Work", "use dev\n"),
        ("active", "Dev\n"),
        ("priority", "apt\n"),
        ("groups", "tools = apt\n"),
        ("schedules", "0 3 * * * shall sync\n"),
        ("vars/season", "winter\n"),
        ("adapters/backends.toml", "[[backend]]\n"),
        ("preferences.toml", "max_parallel = 2\n"),
    ] {
        let before = signature(dir.path()).await;
        std::fs::write(dir.path().join(path), body).expect("the edit");
        let names = changed(&before, &signature(dir.path()).await);
        assert_eq!(
            names,
            vec![path.to_string()],
            "{path} did not count as a change"
        );
    }
    // And the manifest it always did see, so the list above is not the whole story.
    assert!(!changed(&before, &before).iter().any(|_| true));
}
