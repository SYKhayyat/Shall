use super::*;

/// And the openers are real: one file in, one file out, per suffix.
///
/// The table check above passes on a table; this passes only if the bytes come back. Both
/// are here because a wrong codec in the table (`.txz` mapped to gzip, say) satisfies the
/// first and nothing else.
#[test]
fn every_offered_archive_actually_round_trips() {
    use std::io::Write;

    let dir = tempfile::tempdir().unwrap();
    let payload = b"shall-was-here";

    // One tar in memory, wrapped a different way per suffix.
    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        let mut header = tar::Header::new_gnu();
        header.set_size(payload.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "payload.txt", &payload[..])
            .unwrap();
        builder.finish().unwrap();
    }

    for suffix in [
        ".tar.gz", ".tgz", ".tar.xz", ".txz", ".tar.bz2", ".tbz2", ".tar.zst", ".tar",
    ] {
        let wrapped: Vec<u8> = match suffix {
            ".tar.gz" | ".tgz" => {
                let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
                e.write_all(&tar_bytes).unwrap();
                e.finish().unwrap()
            }
            ".tar.xz" | ".txz" => {
                let mut e = xz2::write::XzEncoder::new(Vec::new(), 1);
                e.write_all(&tar_bytes).unwrap();
                e.finish().unwrap()
            }
            ".tar.bz2" | ".tbz2" => {
                let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::fast());
                e.write_all(&tar_bytes).unwrap();
                e.finish().unwrap()
            }
            ".tar.zst" => zstd::stream::encode_all(&tar_bytes[..], 1).unwrap(),
            ".tar" => tar_bytes.clone(),
            other => panic!("no writer for {}", other),
        };

        let archive = dir.path().join(format!("asset{}", suffix));
        fs::write(&archive, &wrapped).unwrap();
        let dest = dir.path().join(format!("out{}", suffix.replace('.', "_")));

        extract_archive(&archive, &dest)
            .unwrap_or_else(|e| panic!("`{}` did not extract: {}", suffix, e));

        let out = dest.join("payload.txt");
        assert!(
            out.exists(),
            "`{}` produced no `payload.txt` — this is the `.tar.zst` bug: the file is \
             accepted, nothing is unpacked, and the caller finds an empty directory",
            suffix
        );
        assert_eq!(
            fs::read(&out).unwrap(),
            payload,
            "`{}` unpacked wrong",
            suffix
        );
    }
}
