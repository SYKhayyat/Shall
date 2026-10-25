use super::*;

#[tokio::test]
async fn first_bad_finds_transition() {
    // good,good,good,BAD,BAD  -> first bad = 3
    assert_eq!(over(&[true, true, true, false, false]).await, Some(3));
}
