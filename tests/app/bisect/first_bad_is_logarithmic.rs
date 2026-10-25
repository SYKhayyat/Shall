use super::*;

#[tokio::test]
async fn first_bad_is_logarithmic() {
    // Ensure the oracle is called O(log n) times, not O(n).
    let n = 1024;
    let mut calls = 0;
    let idx = first_bad(n, |i| {
        calls += 1;
        async move { Ok(i < 700) }
    })
    .await
    .unwrap();
    assert_eq!(idx, Some(700));
    assert!(
        calls <= 11,
        "expected <= ceil(log2(1024))+1 calls, got {}",
        calls
    );
}
