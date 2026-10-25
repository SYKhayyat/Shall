use super::*;

#[test]
fn a_handoff_lock_survives_a_toml_round_trip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("github.toml");
    let mut led = ArtifactLedger::new();
    led.record(
        "sharkdp/fd",
        vec![system_lock("fd_10.2.0_amd64.deb", "rpm", "fd")],
    );
    led.save(&path).unwrap();
    let back = ArtifactLedger::load(&path).unwrap();
    let l = &back.locked("sharkdp/fd")[0];
    assert_eq!(l.installed_by.as_deref(), Some("rpm"));
    assert_eq!(l.system_package.as_deref(), Some("fd"));
}
