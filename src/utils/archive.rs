use crate::backends::artifact::format::{Codec, Format, Opener};
use crate::core::{Error, Result};
use std::fs;
use std::path::Path;
use tracing::debug;

/// The largest an archive may expand to on disk, by default. `max_download_bytes` sizes the
/// compressed bytes; nothing bounded what `tar`/`zip` wrote *out* of them, so a 40 MB zstd
/// whose members declare 200 GB filled the disk mid-install — after every download bound had
/// been honoured. Generous: real toolchains unpack to a few hundred MB. `0` removes the bound.
pub const DEFAULT_MAX_UNPACKED_BYTES: u64 = 8 * 1024 * 1024 * 1024;
static MAX_UNPACKED_BYTES: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(DEFAULT_MAX_UNPACKED_BYTES);

pub fn set_max_unpacked_bytes(v: u64) {
    MAX_UNPACKED_BYTES.store(v, std::sync::atomic::Ordering::SeqCst);
}

fn max_unpacked_bytes() -> u64 {
    MAX_UNPACKED_BYTES.load(std::sync::atomic::Ordering::SeqCst)
}

pub fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    if !dest_dir.exists() {
        crate::utils::file::ensure_dir(dest_dir)?;
    }

    let file = fs::File::open(archive_path).map_err(Error::from)?;
    let name = archive_path.to_string_lossy();

    debug!("Extracting archive: {:?} into {:?}", archive_path, dest_dir);

    // **The list that chose the file is the list that opens it.** This was an `if`-chain over
    // four suffixes with `fs::copy` underneath, while `Format` offered six as tarballs — so a
    // `.tar.zst` was selected, downloaded, copied whole into the destination, searched for an
    // executable, found to have none, and reported as a successful install. See
    // `Format::opener_for`.
    match Format::opener_for(&name) {
        Some(Opener::Tar(codec)) => {
            let reader: Box<dyn std::io::Read> = match codec {
                Codec::Gzip => Box::new(flate2::read::GzDecoder::new(file)),
                Codec::Xz => Box::new(xz2::read::XzDecoder::new(file)),
                Codec::Bzip2 => Box::new(bzip2::read::BzDecoder::new(file)),
                Codec::Zstd => Box::new(
                    zstd::stream::read::Decoder::new(file)
                        .map_err(|e| Error::Other(format!("zstd error: {}", e)))?,
                ),
                Codec::Plain => Box::new(file),
            };
            // **Link targets are inside the archive too, and they are trusted by nobody.**
            // `unpack` refuses entry *names* that escape the destination but follows link
            // *targets* as given — so `ln -s /usr/bin/sh ./x` in a tarball plants a symlink
            // the executable search happily walks straight through. Every entry's target is
            // checked against the destination before anything is linked.
            //
            // And every entry's DECLARED size counts against the expansion bound before its
            // bytes are written: the headers state the uncompressed size, so the bomb is seen
            // in full before the first megabyte of it lands.
            let mut archive = tar::Archive::new(reader);
            let canonical_base =
                fs::canonicalize(dest_dir).unwrap_or_else(|_| dest_dir.to_path_buf());
            let cap = max_unpacked_bytes();
            let mut expanded: u64 = 0;
            for entry in archive.entries().map_err(Error::from)? {
                let mut entry = entry.map_err(Error::from)?;
                if cap > 0 {
                    expanded = expanded.saturating_add(entry.size());
                    if expanded > cap {
                        return Err(Error::Other(format!(
                            "archive expands past the {}-byte unpacked bound (declared total \
                             exceeds it at entry {:?}); raise `[config] max_unpacked_bytes` to \
                             allow more",
                            cap,
                            entry.path().map_err(Error::from)?
                        )));
                    }
                }
                if let Some(link) = entry.link_name().map_err(Error::from)? {
                    let link_path = dest_dir
                        .join(entry.path().map_err(Error::from)?)
                        .parent()
                        .unwrap_or(dest_dir)
                        .join(&*link);
                    // Resolvable now: it must land inside. Not resolvable yet (created later
                    // in this same extraction, or a deliberate dangle): an absolute target
                    // can only leave the destination, and a relative one is checked
                    // textually for `../` escapes before its components exist.
                    match fs::canonicalize(&link_path) {
                        Ok(resolved) => {
                            if !resolved.starts_with(&canonical_base) {
                                return Err(Error::Other(format!(
                                    "archive entry {:?} links to {:?}, which leaves the destination",
                                    entry.path().map_err(Error::from)?,
                                    link
                                )));
                            }
                        }
                        Err(_) => {
                            if link.is_absolute() || !link_path.starts_with(&canonical_base) {
                                return Err(Error::Other(format!(
                                    "archive entry {:?} links to {:?}, which leaves the destination",
                                    entry.path().map_err(Error::from)?,
                                    link
                                )));
                            }
                        }
                    }
                }
                entry.unpack_in(dest_dir).map_err(Error::from)?;
            }
        }
        Some(Opener::Zip) => {
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| Error::Other(format!("Zip error: {}", e)))?;
            // Same expansion bound as the tar side: zip's central directory declares every
            // member's uncompressed size up front, so the whole bomb is visible before the
            // first byte is written.
            let cap = max_unpacked_bytes();
            if cap > 0 {
                let declared: u64 = (0..archive.len())
                    .map(|i| match archive.by_index(i) {
                        Ok(f) => f.size(),
                        Err(_) => 0,
                    })
                    .sum();
                if declared > cap {
                    return Err(Error::Other(format!(
                        "zip expands past the {}-byte unpacked bound (members declare {} bytes)",
                        cap, declared
                    )));
                }
            }
            archive
                .extract(dest_dir)
                .map_err(|e| Error::Other(format!("Zip extraction failed: {}", e)))?;
        }
        // Not an archive at all — a bare binary, a `.deb`, an `.exe`. Placing it is the whole
        // job, and `github.rs` calls this for every artifact precisely so that one code path
        // handles both. This is the *only* honest reason to land here: a name `Format` calls
        // an archive and has no opener for cannot reach it, which
        // `an_offered_archive_has_an_opener` is what makes true.
        None => {
            let filename = archive_path
                .file_name()
                .ok_or_else(|| Error::Other("Invalid archive filename".into()))?;
            let target = dest_dir.join(filename);
            fs::copy(archive_path, target).map_err(Error::from)?;
        }
    }

    Ok(())
}

/// Compress a directory tree into a single `.tar.gz` file. The archive stores paths relative
/// to `src_dir` under `root_name/…`, so unpacking recreates one self-contained top folder
/// (the mirror of [`extract_archive`]). Returns the number of bytes written.
pub fn create_tar_gz(src_dir: &Path, dest_file: &Path, root_name: &str) -> Result<u64> {
    use flate2::write::GzEncoder;
    use flate2::Compression;

    if let Some(parent) = dest_file.parent() {
        if !parent.as_os_str().is_empty() {
            crate::utils::file::ensure_dir(parent)?;
        }
    }
    let out = fs::File::create(dest_file).map_err(Error::from)?;
    let enc = GzEncoder::new(out, Compression::default());
    let mut builder = tar::Builder::new(enc);
    builder
        .append_dir_all(root_name, src_dir)
        .map_err(Error::from)?;
    // Finish the tar, then finish the gzip stream, so all bytes are flushed to disk.
    let enc = builder.into_inner().map_err(Error::from)?;
    enc.finish().map_err(Error::from)?;
    let size = fs::metadata(dest_file).map(|m| m.len()).unwrap_or(0);
    debug!("Wrote {} bytes to {:?}", size, dest_file);
    Ok(size)
}

// `is_archive` lived here too — a fifth copy of the extension list, with zero callers, whose
// five entries had already fallen behind `Format`'s six. `Format::is_archive` answers the same
// question from the table that selects the file, and is the one that is asked.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::artifact::format::Format;

    /// **The gate F-7 is really about.** `Format` offered `.tar.zst` and `.txz` as tarballs and
    /// nothing could open either, so the selector and the extractor disagreed in silence: the
    /// asset was chosen, downloaded, `fs::copy`d whole into the destination, searched for an
    /// executable, found to have none — and the install reported success having deployed
    /// nothing. A check drawn around `extract_archive`'s own four suffixes could never have
    /// seen it, because the missing suffixes were in the *other* list.
    #[test]
    fn every_archive_the_selector_offers_has_an_opener() {
        let offered: Vec<&str> = Format::ALL
            .into_iter()
            .filter(|f| f.is_archive())
            .flat_map(|f| f.suffixes().iter().copied())
            .collect();

        // The instrument first: a scan that enumerates nothing passes silently, and this one
        // is reading a table it could just as easily read as empty.
        assert!(
            offered.len() >= 8,
            "the scan found {} archive suffixes, which is fewer than the tarball list alone \
             holds — it is reading the wrong thing",
            offered.len()
        );
        assert!(offered.contains(&".tar.zst"), "{:?}", offered);

        for suffix in offered {
            assert!(
                Format::opener_for(&format!("thing{}", suffix)).is_some(),
                "`{}` is offered as an archive and nothing can open it — which is the exact \
                 shape that made a `.tar.zst` install report success having deployed nothing",
                suffix
            );
        }
    }

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
                    let mut e =
                        flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
                    e.write_all(&tar_bytes).unwrap();
                    e.finish().unwrap()
                }
                ".tar.xz" | ".txz" => {
                    let mut e = xz2::write::XzEncoder::new(Vec::new(), 1);
                    e.write_all(&tar_bytes).unwrap();
                    e.finish().unwrap()
                }
                ".tar.bz2" | ".tbz2" => {
                    let mut e =
                        bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::fast());
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

    /// And an in-bounds link — the legitimate shape, a `bin/tool -> ../lib/tool` layout —
    /// still extracts, so the refusal is aimed and not a ban on symlinks.
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
}
