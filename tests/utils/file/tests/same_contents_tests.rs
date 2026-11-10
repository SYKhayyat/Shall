use super::*;

/// The two questions this answers, and the one it must not get wrong.
///
/// `link:` asks it on every sync for every declared `link:` — a copy it made is "already in
/// effect", and answering wrongly backs the file up as `<target>.shall-backup` and rewrites it
/// under a summary reading `already up to date`. The shim check asks it of a deployed copy.
/// Both used to read both files whole, so a 348 MB debug build was loaded twice per question.
#[tokio::test]
async fn equal_and_unequal_copies_are_answered_without_reading_either_whole() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");

    tokio::fs::write(&a, b"same bytes").await.unwrap();
    tokio::fs::write(&b, b"same bytes").await.unwrap();
    assert!(same_contents(&a, &b).await, "identical copies must match");

    // Same length, one byte different: the size cannot settle it, and the chunk that differs
    // is the first one.
    tokio::fs::write(&b, b"same byteS").await.unwrap();
    assert!(
        !same_contents(&a, &b).await,
        "a same-length difference was read as a match"
    );

    // Different lengths, and the difference is in a later chunk than the first read.
    let long: Vec<u8> = std::iter::repeat(b'x').take(200 * 1024).collect();
    tokio::fs::write(&a, &long).await.unwrap();
    let mut shorter = long.clone();
    shorter.truncate(150 * 1024);
    tokio::fs::write(&b, &shorter).await.unwrap();
    assert!(
        !same_contents(&a, &b).await,
        "a file that ends early was read as a match"
    );

    // A multi-chunk file that matches everywhere, including the last partial block.
    let other = dir.path().join("c");
    tokio::fs::write(&other, &long).await.unwrap();
    assert!(
        same_contents(&a, &other).await,
        "a multi-chunk copy that matches was read as a difference"
    );

    // A file that is not there cannot be the same as one that is.
    assert!(
        !same_contents(&a, &dir.path().join("missing")).await,
        "a missing file must not compare equal to an existing one"
    );
}
