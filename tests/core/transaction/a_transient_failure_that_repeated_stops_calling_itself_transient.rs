use super::*;

#[test]
fn a_transient_failure_that_repeated_stops_calling_itself_transient() {
    let out = falsify_transience(transient("`luarocks` failed: Failed downloading â€¦"), 3);
    assert_eq!(out.retryability(), Retryability::Exhausted);
    assert!(
        out.to_string().contains("did not change"),
        "the message must say what was tried: {out}"
    );
}
