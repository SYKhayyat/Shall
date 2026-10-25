use super::*;

#[test]
fn an_entry_survives_a_round_trip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("locks").join("github.toml");
    let mut led = ArtifactLedger::new();
    led.record("sharkdp/fd", vec![lock("fd.tar.gz", Some("abc123"))]);
    led.save(&path).unwrap();

    let back = ArtifactLedger::load(&path).unwrap();
    let [entry] = back.locked("sharkdp/fd") else {
        panic!("expected one locked artifact");
    };
    assert_eq!(entry.asset, "fd.tar.gz");
    assert_eq!(entry.url, "https://example.invalid/fd.tar.gz");
    assert_eq!(entry.format, "tarball");
    assert_eq!(entry.sha256.as_deref(), Some("abc123"));
}
