use super::*;

#[tokio::test]
async fn first_bad_empty_is_none() {
    assert_eq!(over(&[]).await, None);
}
