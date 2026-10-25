use super::*;

#[tokio::test]
async fn two_managers_do_not_share_one_answer() {
    let memo = InstalledListings::new();
    let apt = memo
        .once("apt", async { Ok(vec![pkg("jq")]) })
        .await
        .unwrap();
    let npm = memo
        .once("npm", async { Ok(vec![pkg("prettier"), pkg("eslint")]) })
        .await
        .unwrap();
    assert_eq!(apt.len(), 1);
    assert_eq!(npm.len(), 2);
}
