use super::*;

/// The backoff doubles from the first retry, and the cap is a cap.
///
/// Called only with `attempt >= 2`, so attempt 2 â€” the first retry â€” waits `initial`
/// exactly. Each value below is one the shift, the exponent or the multiplication would
/// get wrong on its own.
#[test]
fn the_backoff_doubles_from_the_first_retry_and_stops_at_the_cap() {
    let initial = Duration::from_millis(500);
    let max = Duration::from_secs(30);

    assert_eq!(
        backoff_for(2, initial, max),
        Duration::from_millis(500),
        "the first retry waits the initial backoff, undoubled"
    );
    assert_eq!(backoff_for(3, initial, max), Duration::from_secs(1));
    assert_eq!(backoff_for(4, initial, max), Duration::from_secs(2));
    assert_eq!(
        backoff_for(7, initial, max),
        Duration::from_secs(16),
        "still doubling while it is under the cap"
    );
    assert_eq!(
        backoff_for(8, initial, max),
        max,
        "thirty-two seconds is over a thirty-second cap, so the cap is what is waited"
    );
}
