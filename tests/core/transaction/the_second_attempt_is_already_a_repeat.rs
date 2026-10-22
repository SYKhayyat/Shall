use super::*;

/// Two attempts is the experiment; one is not.
///
/// The tests above use three attempts and one, and `attempts < 2` reads `<= 2` without
/// either of them noticing â€” which quietly raises the bar to three, so the *second*
/// identical failure stops counting as evidence. Two is the smallest number of attempts
/// that can show a failure repeating, and it is the only value at which the two spellings
/// disagree.
#[test]
fn the_second_attempt_is_already_a_repeat() {
    let out = falsify_transience(transient("`gem` failed: Connection reset"), 2);
    assert_eq!(
        out.retryability(),
        Retryability::Exhausted,
        "a failure seen twice has been tested once, which is the whole experiment"
    );
    assert!(
        out.to_string().contains("tried 2 times"),
        "the message must say how many attempts stand behind it: {out}"
    );
}
