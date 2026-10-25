use super::*;

#[test]
fn forgetting_a_package_drops_its_entry() {
    // A lock left behind after a removal would pin the next install to an artifact
    // chosen for a declaration that no longer exists.
    let mut led = ArtifactLedger::new();
    led.record("sharkdp/fd", vec![lock("fd.tar.gz", None)]);
    assert!(led.forget("sharkdp/fd"));
    assert!(!led.forget("sharkdp/fd"));
    assert!(led.is_empty());
}
