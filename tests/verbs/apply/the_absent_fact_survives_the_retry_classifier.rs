use super::*;

/// The retry loop must not launder the fact. A failure that travels through
/// `falsify_transience` and back is still about a name that is not there.
#[test]
fn the_absent_fact_survives_the_retry_classifier() {
    let e = boxed(Error::command_failed_absent("`npm` failed: 404 Not Found"));
    assert!(says_a_name_is_absent(&e));
    assert_eq!(
        e.downcast_ref::<Error>().map(|x| x.retryability()),
        Some(Retryability::Permanent),
        "an absent name is not worth retrying, so it never enters the loop"
    );
}
