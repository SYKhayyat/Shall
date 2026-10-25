use super::*;

/// An unreadable file is an error, not silently empty. The missing-file rule is about
/// *absence*; a file that exists and cannot be parsed is a fact the user needs told, and
/// swallowing it would silently discard every recorded expansion, approval and pin.
#[test]
fn a_corrupt_file_is_an_error_and_not_an_empty_ledger() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("probe.toml");
    std::fs::write(&path, "this is not toml { [ ").unwrap();
    let err = Probe::load(&path).unwrap_err();
    assert!(
        matches!(err, Error::Toml(_)),
        "a corrupt ledger must not read as empty; got {err:?}"
    );
}
