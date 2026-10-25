use super::*;

/// An unknown word is named back, and the message carries the whole vocabulary — a refusal
/// that says only "unknown" makes the reader go and find the list.
#[test]
fn an_unknown_word_is_refused_with_the_list_of_real_ones() {
    for bad in ["firewall", "version", "script", "packagess"] {
        let err = LockSelection::parse(bad, &[]).expect_err(bad);
        let msg = err.to_string();
        assert!(msg.contains(bad), "it must quote the word back: {msg}");
        assert!(msg.contains("everything"), "the groups are missing: {msg}");
        assert!(msg.contains("adapters"), "the kinds are missing: {msg}");
    }
    // And on the exclusion side, which is a separate parse and was the half that got
    // forgotten when this was two code paths.
    let err = LockSelection::parse("everything", &words(&["firewal"])).expect_err("typo");
    assert!(err.to_string().contains("firewal"), "{err}");
    assert!(err.to_string().contains("--except"), "{err}");
}
