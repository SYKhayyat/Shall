use super::*;

#[tokio::test]
async fn first_bad_all_good_is_none() {
    assert_eq!(over(&[true, true, true]).await, None);
}
