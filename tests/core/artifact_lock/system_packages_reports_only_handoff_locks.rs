use super::*;

#[test]
fn system_packages_reports_only_handoff_locks() {
    // D5: a `.deb` handed to dpkg records the installer and the name it listed the package
    // under; a plain PATH-deployed artifact contributes nothing to the dedup set.
    let mut led = ArtifactLedger::new();
    led.record(
        "sharkdp/fd",
        vec![system_lock("fd_10.2.0_amd64.deb", "dpkg", "fd")],
    );
    led.record("owner/plain", vec![lock("plain.tar.gz", Some("z9"))]);
    let owned = led.system_packages();
    assert_eq!(owned, vec![("dpkg".to_string(), "fd".to_string())]);
}
