use super::*;

/// The token is the API. A rename that leaves the greppable string behind is a rename that
/// quietly breaks every build log anyone has ever grepped, and nothing else would notice.
#[test]
fn the_marker_carries_a_stable_greppable_token() {
    assert_eq!(UNMEASURED_TOKEN, "shall-latency-unmeasured:");
    let got = captured(|| super::unmeasured("sync", "because"));
    assert_eq!(got.len(), 1);
    assert!(
        got[0].contains(UNMEASURED_TOKEN),
        "the token is the contract: {:?}",
        got[0]
    );
}
