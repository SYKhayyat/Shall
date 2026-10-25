use super::*;

#[tokio::test]
async fn first_bad_all_bad_is_zero() {
    assert_eq!(over(&[false, false, false]).await, Some(0));
}
