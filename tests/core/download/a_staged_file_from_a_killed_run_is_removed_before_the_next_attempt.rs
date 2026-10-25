use super::*;

/// A staged file left by a kill is cleaned on the next attempt rather than left to rot beside
/// the artifact — the one exit `Drop` cannot cover, handled where the name is known.
#[tokio::test]
async fn a_staged_file_from_a_killed_run_is_removed_before_the_next_attempt() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dest = dir.path().join("tool");
    let leftover = part_path(&dest);
    std::fs::write(&leftover, b"half a download from a run that was killed").expect("seeded");

    let staged = stage_capped_to(undeclared("the whole thing"), &dest, "a tool", None)
        .await
        .expect("the retry");
    assert_eq!(
        std::fs::read(staged.path()).expect("the staged bytes"),
        b"the whole thing",
        "the retry staged the leftover from the killed run instead of replacing it"
    );
    staged.commit().await.expect("committed");
    assert!(!leftover.exists());
}
