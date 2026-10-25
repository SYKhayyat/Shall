use super::*;

/// And an in-bounds link — the legitimate shape, a `bin/tool -> ../lib/tool` layout —
/// still extracts, so the refusal is aimed and not a ban on symlinks.
#[cfg(unix)]
#[test]
fn a_hard_link_that_leaves_the_destination_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        let mut header = tar::Header::new_gnu();
        header.set_size(0);
        header.set_entry_type(tar::EntryType::Link);
        header.set_mode(0o644);
        builder
            .append_link(&mut header, "bin/tool", "../../outside")
            .unwrap();
        builder.finish().unwrap();
    }
    let archive = dir.path().join("asset.tar");
    fs::write(&archive, &tar_bytes).unwrap();
    let err = extract_archive(&archive, &dir.path().join("out")).unwrap_err();
    assert!(err.to_string().contains("leaves the destination"), "{err}");
}
