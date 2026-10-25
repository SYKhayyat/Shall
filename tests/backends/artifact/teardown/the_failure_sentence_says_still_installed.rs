use super::*;

/// A removal that failed must not read as a removal that succeeded, and the noun is the
/// only part that differs between the three.
#[test]
fn the_failure_sentence_says_still_installed() {
    let e = still_installed("web resource", &["a: denied".into(), "b: busy".into()]);
    let text = e.to_string();
    assert!(
        text.contains("could not remove 2 web resource(s)"),
        "{text}"
    );
    // Both halves: three backends said one or the other, and neither may be dropped.
    assert!(text.contains("still installed"), "{text}");
    assert!(text.contains("still on disk"), "{text}");
    assert!(text.contains("a: denied, b: busy"), "{text}");
}
