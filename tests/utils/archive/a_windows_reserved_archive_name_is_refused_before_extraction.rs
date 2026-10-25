use super::*;

#[cfg(unix)]
#[test]
fn a_windows_reserved_archive_name_is_refused_before_extraction() {
    let dir = tempfile::tempdir().unwrap();
    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        let mut header = tar::Header::new_gnu();
        header.set_size(3);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "NUL.exe", &b"bad"[..])
            .unwrap();
        builder.finish().unwrap();
    }
    let archive = dir.path().join("asset.tar");
    fs::write(&archive, &tar_bytes).unwrap();
    let err = extract_archive(&archive, &dir.path().join("out")).unwrap_err();
    assert!(err.to_string().contains("reserved Windows"), "{err}");
}
