use super::*;

/// helm downloads nothing Shall can see, so the heading cannot claim it did.
#[test]
fn the_heading_does_not_claim_shall_downloaded_it() {
    assert!(!UNVERIFIED_HEADING.contains("downloaded"));
    assert!(UNVERIFIED_HEADING.contains("@unverified"));
}
