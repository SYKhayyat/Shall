use super::*;

#[test]
fn recording_replaces_the_whole_set() {
    // A declaration that now resolves to fewer artifacts must not keep the dropped ones.
    let mut led = ArtifactLedger::new();
    led.record(
        "foo/bar",
        vec![lock("a.tar.gz", None), lock("b.tar.gz", None)],
    );
    led.record("foo/bar", vec![lock("a.tar.gz", None)]);
    assert_eq!(led.locked("foo/bar").len(), 1);
}
