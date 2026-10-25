use super::*;

#[test]
fn signatures_and_checksums_are_not_installable_assets() {
    assert!(Format::is_metadata_filename("fd_10.2.0_amd64.deb.sha256"));
    assert!(Format::is_metadata_filename("checksums.txt"));
    assert!(Format::is_metadata_filename("release.asc"));
    assert!(!Format::is_metadata_filename("fd_10.2.0_amd64.deb"));
    // The whole digest-list family, in the spellings real releases use — rclone ships the
    // first three in one release, and none has an extension to be recognised by (D2).
    for name in [
        "MD5SUMS",
        "SHA1SUMS",
        "SHA256SUMS",
        "sha512sums.txt",
        "sha256sum.txt",
    ] {
        assert!(Format::is_metadata_filename(name), "{}", name);
    }
    // And a package that merely ends in those letters is not one.
    assert!(!Format::is_metadata_filename(
        "consums-1.0-linux-amd64.tar.gz"
    ));
}
