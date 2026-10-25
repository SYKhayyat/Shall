use super::*;

#[tokio::test]
async fn a_mutation_makes_the_next_listing_real() {
    let memo = InstalledListings::new();
    let before = memo.once("apt", async { Ok(vec![]) }).await.unwrap();
    assert!(before.is_empty());
    memo.forget_all();
    let after = memo
        .once("apt", async { Ok(vec![pkg("jq")]) })
        .await
        .unwrap();
    assert_eq!(
        after.len(),
        1,
        "the listing taken before the install was still being served after it"
    );
}
