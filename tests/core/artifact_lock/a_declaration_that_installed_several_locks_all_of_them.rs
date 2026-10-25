use super::*;

#[test]
fn a_declaration_that_installed_several_locks_all_of_them() {
    // `@asset=all` (VIII.2). One entry per declaration could describe only the first.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("locks").join("github.toml");
    let mut led = ArtifactLedger::new();
    led.record(
        "foo/bar",
        vec![
            lock("bar.tar.gz", Some("a1")),
            lock("bar-server.tar.gz", Some("b2")),
        ],
    );
    led.save(&path).unwrap();

    let back = ArtifactLedger::load(&path).unwrap();
    let names: Vec<&str> = back
        .locked("foo/bar")
        .iter()
        .map(|l| l.asset.as_str())
        .collect();
    assert_eq!(names, vec!["bar.tar.gz", "bar-server.tar.gz"]);
}
