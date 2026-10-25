use super::*;

/// **Content, not size and mtime.** The old fingerprint was `(len, mtime)`, and this is the
/// edit it cannot see: the same number of bytes, written now.
#[tokio::test]
async fn a_rewrite_of_the_same_bytes_at_the_same_length_is_a_change() {
    let dir = repo().await;
    std::fs::write(dir.path().join("modules/dev.txt"), "apt:curl\n").expect("same length");
    let before = signature(dir.path()).await;
    std::fs::write(dir.path().join("modules/dev.txt"), "apt:htop\n").expect("same length again");
    assert_eq!(
        changed(&before, &signature(dir.path()).await),
        vec!["modules/dev.txt".to_string()],
        "an edit that preserves the length was invisible to the old fingerprint"
    );
}
