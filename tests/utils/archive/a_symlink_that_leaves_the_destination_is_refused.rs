use super::*;

/// A tarball whose symlink points OUT of the destination is refused, not unpacked: the
/// executable search that follows extraction would walk the link into whatever it names.
#[cfg(unix)]
#[test]
fn a_symlink_that_leaves_the_destination_is_refused() {
    use std::io::Write;

    let dir = tempfile::tempdir().unwrap();
    let outside = dir.path().join("outside-target");
    fs::write(&outside, b"owned").unwrap();

    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        let mut header = tar::Header::new_gnu();
        header.set_size(0);
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_mode(0o777);
        builder.append_link(&mut header, "evil", &outside).unwrap();
        builder.finish().unwrap();
    }
    let archive = dir.path().join("asset.tar.gz");
    {
        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        e.write_all(&tar_bytes).unwrap();
        fs::write(&archive, e.finish().unwrap()).unwrap();
    }

    let dest = dir.path().join("out");
    let err = extract_archive(&archive, &dest).unwrap_err().to_string();
    assert!(
        err.contains("leaves the destination"),
        "an escaping symlink must be refused by name: {err}"
    );
}
