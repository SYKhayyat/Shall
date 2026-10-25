use super::*;

/// The fallback that must stay. `github.rs` hands every artifact to `extract_archive`,
/// including a bare binary and a `.deb`, and placing those is the whole job — so `None`
/// from `opener_for` has to mean *put it down*, not *fail*.
#[test]
fn something_that_is_not_an_archive_is_placed_whole() {
    let dir = tempfile::tempdir().unwrap();
    let bin = dir.path().join("ripgrep");
    fs::write(&bin, b"ELF").unwrap();
    let dest = dir.path().join("out");

    extract_archive(&bin, &dest).unwrap();
    assert_eq!(fs::read(dest.join("ripgrep")).unwrap(), b"ELF");
    assert!(Format::opener_for("ripgrep").is_none());
    assert!(Format::opener_for("thing.deb").is_none());
}
