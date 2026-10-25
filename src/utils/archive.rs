use crate::backends::artifact::format::{Codec, Format, Opener};
use crate::core::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

/// The largest an archive may expand to on disk, by default. `max_download_bytes` sizes the
/// compressed bytes; nothing bounded what `tar`/`zip` wrote *out* of them, so a 40 MB zstd
/// whose members declare 200 GB filled the disk mid-install — after every download bound had
/// been honoured. Generous: real toolchains unpack to a few hundred MB. `0` removes the bound.
pub const DEFAULT_MAX_UNPACKED_BYTES: u64 = 8 * 1024 * 1024 * 1024;
pub static MAX_UNPACKED_BYTES: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(DEFAULT_MAX_UNPACKED_BYTES);

pub fn set_max_unpacked_bytes(v: u64) {
    MAX_UNPACKED_BYTES.store(v, std::sync::atomic::Ordering::SeqCst);
}

fn max_unpacked_bytes() -> u64 {
    MAX_UNPACKED_BYTES.load(std::sync::atomic::Ordering::SeqCst)
}

/// Validate archive names in a platform-independent way. An archive may be downloaded on one
/// platform and extracted on another, so host `Path` parsing alone is insufficient: on Unix a
/// backslash is a filename character, while Windows treats it as a separator.
fn validate_archive_path(path: &Path) -> Result<()> {
    let raw = path.to_string_lossy();
    let normalized = raw.replace('\\', "/");
    if normalized.starts_with('/')
        || normalized.starts_with("//")
        || normalized.as_bytes().get(1).is_some_and(|b| *b == b':')
    {
        return Err(Error::Other(format!(
            "archive entry {:?} is an absolute path",
            path
        )));
    }

    const RESERVED: [&str; 22] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let mut depth = 0usize;
    for component in normalized.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if depth == 0 {
                    return Err(Error::Other(format!(
                        "archive entry {:?} leaves the destination",
                        path
                    )));
                }
                depth -= 1;
            }
            value => {
                let stem = value
                    .split('.')
                    .next()
                    .unwrap_or(value)
                    .trim_end_matches(' ');
                if RESERVED.iter().any(|name| stem.eq_ignore_ascii_case(name)) {
                    return Err(Error::Other(format!(
                        "archive entry {:?} uses the reserved Windows device name `{}`",
                        path, value
                    )));
                }
                depth += 1;
            }
        }
    }
    Ok(())
}

fn validate_archive_link(entry: &Path, link: &Path) -> Result<()> {
    validate_archive_path(entry)?;
    let raw = link.to_string_lossy();
    let normalized = raw.replace('\\', "/");
    if normalized.starts_with('/')
        || normalized.starts_with("//")
        || normalized.as_bytes().get(1).is_some_and(|b| *b == b':')
    {
        return Err(Error::Other(format!(
            "archive entry {:?} links to {:?}, which leaves the destination",
            entry, link
        )));
    }

    let parent = entry.parent().unwrap_or_else(|| Path::new(""));
    let target = parent.join(link);
    validate_archive_path(&target).map_err(|_| {
        Error::Other(format!(
            "archive entry {:?} links to {:?}, which leaves the destination",
            entry, link
        ))
    })
}

/// Tar hard-link names are resolved from the archive root, while symlink names are resolved
/// relative to the link's containing directory. Keep the two rules explicit so a safe hard link
/// is not rejected and a `../` target cannot escape through a nested member path.
fn validate_archive_hard_link(entry: &Path, link: &Path) -> Result<()> {
    validate_archive_path(entry)?;
    validate_archive_path(link).map_err(|_| {
        Error::Other(format!(
            "archive entry {:?} links to {:?}, which leaves the destination",
            entry, link
        ))
    })
}

/// **The unpacked-size total, summed so it cannot pass the bound by wrapping.**
///
/// Both archive formats ask the same question — *how big does this expand to?* — and both must
/// answer it with arithmetic that cannot be defeated by the archive itself. The tar side walked
/// its entries with `saturating_add`; the zip side called `.sum()` over the members' declared
/// sizes, and **`Iterator::sum` for integers wraps in a release build** (and panics in a debug
/// one). Every member's size is a field the archive's author wrote, so a crafted zip declaring
/// `u64::MAX` and `2` summed to `1`, sat comfortably under any `max_unpacked_bytes`, and the bomb
/// check — the one thing standing between a download and an unbounded write — said yes. That is
/// `PLAN.md` #86, and the tar branch is the fixed twin this was measured against.
///
/// One named rule rather than an operator spelled in each branch, so the two sides cannot drift
/// apart again: a second way to total a size is how the raw-vs-rendered pair in the planner
/// happened (`PLAN.md` #91). The tar branch accumulates as it walks — it has to, to stop before
/// writing the entry that passes the bound — and the zip branch has every size up front, so the
/// fold is over a collection. Both go through `add_unpacked`.
pub fn add_unpacked(total: u64, size: u64) -> u64 {
    total.saturating_add(size)
}

pub fn declared_unpacked_total(sizes: impl IntoIterator<Item = u64>) -> u64 {
    sizes.into_iter().fold(0u64, add_unpacked)
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
            // the executable search happily walks straight through.
            //
            // The containment test is LEXICAL, deliberately: canonicalize() here answered
            // differently per platform (macOS resolves `/var` → `/private/var`, so the base
            // and the target disagreed about the prefix and an in-bounds link was refused).
            // Absolute targets are refused outright; relative ones may climb with `..` only
            // as far as the destination root. A relative link that lands inside and then
            // points at ANOTHER in-dest link is not chased here — each entry is checked when
            // its own turn comes, which bounds one hop per archive entry.
            let mut archive = tar::Archive::new(reader);
            let cap = max_unpacked_bytes();
            let mut expanded: u64 = 0;
            for entry in archive.entries().map_err(Error::from)? {
                let mut entry = entry.map_err(Error::from)?;
                if cap > 0 {
                    expanded = add_unpacked(expanded, entry.size());
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
                let entry_path = entry.path().map_err(Error::from)?.into_owned();
                validate_archive_path(&entry_path)?;
                if let Some(link) = entry.link_name().map_err(Error::from)? {
                    // Validate symlinks and hard links alike. `unpack_in` confines the member
                    // name, but a link target is a second path that can be followed later by
                    // executable discovery. Tar hard links are archive-root-relative; symlinks
                    // are relative to the member's parent directory.
                    if entry.header().entry_type().is_hard_link() {
                        validate_archive_hard_link(&entry_path, &link)?;
                    } else {
                        validate_archive_link(&entry_path, &link)?;
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
                let sizes = (0..archive.len()).map(|i| match archive.by_index(i) {
                    Ok(f) => f.size(),
                    Err(_) => 0,
                });
                let declared: u64 = declared_unpacked_total(sizes);
                if declared > cap {
                    return Err(Error::Other(format!(
                        "zip expands past the {}-byte unpacked bound (members declare {} bytes)",
                        cap, declared
                    )));
                }
            }
            // `ZipArchive::extract` sanitizes member names for the host platform, but ZIP files
            // can carry Unix symlinks and Windows separators are ordinary characters on Unix.
            // Inspect every member before extracting any of them so a later link cannot escape
            // after an earlier member has already been written.
            for i in 0..archive.len() {
                let mut member = archive
                    .by_index(i)
                    .map_err(|e| Error::Other(format!("Zip member {}: {}", i, e)))?;
                // `zip` 9 made the name fallible: a member whose name is not valid UTF-8 is
                // an error rather than a lossy replacement, because the lossy string could
                // name a different path than the one being extracted.
                let reported = member.name().map_err(|e| {
                    Error::Other(format!("Zip member {} has no usable name: {}", i, e))
                })?;
                let name = PathBuf::from(&*reported);
                validate_archive_path(&name)?;
                if member.is_symlink() {
                    use std::io::Read;
                    let mut target = String::new();
                    member.read_to_string(&mut target).map_err(|e| {
                        Error::Other(format!("ZIP symlink {:?} cannot be read: {}", name, e))
                    })?;
                    validate_archive_link(&name, Path::new(&target))?;
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
        // `every_archive_the_selector_offers_has_an_opener` is what makes true.
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
