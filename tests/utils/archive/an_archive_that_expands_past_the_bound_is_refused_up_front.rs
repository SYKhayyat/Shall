use super::*;

/// A tarball whose members declare more than the expansion bound is refused before the
/// first byte is written — the compressed bytes passed every download bound; this is the
/// other half.
#[test]
fn an_archive_that_expands_past_the_bound_is_refused_up_front() {
    let dir = tempfile::tempdir().unwrap();
    // ~64 KiB of declared size: tiny, but over a bound set to 1000 for the test.
    let payload = vec![b'z'; 64 * 1024];

    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        let mut header = tar::Header::new_gnu();
        header.set_size(payload.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "big", payload.as_slice())
            .unwrap();
        builder.finish().unwrap();
    }
    let archive = dir.path().join("asset.tar");
    fs::write(&archive, &tar_bytes).unwrap();

    let previous = MAX_UNPACKED_BYTES.load(std::sync::atomic::Ordering::SeqCst);
    set_max_unpacked_bytes(1_000);
    let dest = dir.path().join("out");
    let err = extract_archive(&archive, &dest).unwrap_err().to_string();
    set_max_unpacked_bytes(previous);

    assert!(
        err.contains("unpacked bound"),
        "the refusal names the bound and the knob: {err}"
    );
    assert!(
        !dest.join("big").exists(),
        "nothing was written before the refusal"
    );
}
