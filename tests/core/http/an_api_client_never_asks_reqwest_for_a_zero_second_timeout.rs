use super::*;

#[test]
fn an_api_client_never_asks_reqwest_for_a_zero_second_timeout() {
    // reqwest reads a zero-second timeout as "fail instantly", not "no bound", so an API
    // caller handed a configured 0 must not pass it through.
    let _ = api("shall-pool-d", 0).unwrap();
    let _ = api("shall-pool-d", 1).unwrap();
    assert_eq!(
        mine("shall-pool-d"),
        1,
        "a 0-second API timeout was not raised to 1 — it built a distinct, \
         instantly-failing client"
    );
}
