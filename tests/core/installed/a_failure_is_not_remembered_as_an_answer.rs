use super::*;

#[tokio::test]
async fn a_failure_is_not_remembered_as_an_answer() {
    let memo = InstalledListings::new();
    let first = memo
        .once("apt", async {
            Err(shall::core::Error::Other("the index was locked".into()))
        })
        .await;
    assert!(first.is_err());
    let second = memo
        .once("apt", async { Ok(vec![pkg("jq")]) })
        .await
        .unwrap();
    assert_eq!(
        second.len(),
        1,
        "one transient failure became the run's permanent answer"
    );
}
