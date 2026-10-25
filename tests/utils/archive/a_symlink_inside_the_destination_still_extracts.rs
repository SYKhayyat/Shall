use super::*;

#[cfg(unix)]
#[test]
fn a_symlink_inside_the_destination_still_extracts() {
    let dir = tempfile::tempdir().unwrap();
    let payload = b"real";

    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        let mut header = tar::Header::new_gnu();
        header.set_size(payload.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "lib/real", &payload[..])
            .unwrap();

        let mut link_header = tar::Header::new_gnu();
        link_header.set_size(0);
        link_header.set_entry_type(tar::EntryType::Symlink);
        link_header.set_mode(0o777);
        builder
            .append_link(&mut link_header, "bin/tool", "../lib/real")
            .unwrap();
        builder.finish().unwrap();
    }
    let archive = dir.path().join("asset.tar");
    fs::write(&archive, &tar_bytes).unwrap();

    let dest = dir.path().join("out");
    extract_archive(&archive, &dest).unwrap();
    assert_eq!(fs::read(dest.join("lib/real")).unwrap(), payload);
}
