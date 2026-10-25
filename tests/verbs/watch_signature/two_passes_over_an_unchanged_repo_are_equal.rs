use super::*;

/// The signature is sorted and unique, so two passes are equal iff nothing moved — the
/// property the comparison rests on, asserted on the map itself rather than on the diff.
#[tokio::test]
async fn two_passes_over_an_unchanged_repo_are_equal() {
    let dir = repo().await;
    assert_eq!(signature(dir.path()).await, signature(dir.path()).await);
    let taken = signature(dir.path()).await;
    let first: Vec<&String> = taken.keys().collect();
    let mut sorted = first.clone();
    sorted.sort();
    assert_eq!(first, sorted, "the walk order leaked into the signature");
}
