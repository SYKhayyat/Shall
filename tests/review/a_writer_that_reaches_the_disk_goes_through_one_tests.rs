//! **`utils/file.rs` opened with "There were two of these… so there is now one." There were
//! four.**
//!
//! `atomic_write` fsyncs. `CommandExecutor::write_atomic` did not — no `flush`, no `sync_all` —
//! and it is what writes a systemd unit and a `link:` target. `CommandExecutor::write_secret`
//! flushed and did not fsync. `InstalledListings::save_to_disk` writes and renames with neither,
//! deliberately, because it is a cache.
//!
//! **A rename is atomic against a concurrent reader and says nothing about power loss.** The
//! directory entry can reach the disk before the bytes it points at do, which leaves a file of
//! the right name and zero length. So a crash after a sync could leave an empty systemd unit
//! while `registry.json` and the WAL — which went through `persist` — survived intact: Shall's
//! record of what it did, without the thing it did.
//!
//! This scan is the half a comment cannot do. The durability is one function now
//! ([`shall::utils::file::durable_write`], private to the crate); what stays plural is the
//! *preview policy*, deliberately, and this enumerates both so a third cannot appear unnoticed.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::ledger::Ledger;

/// The files allowed to hold a rename-into-place of their own, and why.
///
/// Two are the two front doors:
///
/// - `utils/file.rs` is `durable_write` itself, plus `persist`'s preview policy for the config
///   repo: *print `would write …` and stop*.
/// - `core/installed.rs` is the installed-listing cache, which is **correctly** neither durable
///   nor preview-aware: a torn cache file is a cache miss, an fsync per listing would be a disk
///   barrier on the read path, and its temp name carries the pid because two `shall` runs
///   sharing one temp path is the torn listing the mechanism exists to prevent.
///
/// And one is a third door with a reason that is an impossibility rather than a preference: a
/// streamed download body has no buffer to hand `durable_write`, and buffering one is the defect
/// II.55 exists to remove.
const MAY_RENAME: &[(&str, &str)] = &[
    (
        "src/utils/file.rs",
        "`durable_write` is the one durable write, and `persist` is the config repo's preview \
         policy in front of it.",
    ),
    (
        "src/core/installed.rs",
        "The installed-listing cache. A torn file is a cache miss, so it is deliberately not \
         durable; its temp name carries the pid because the rename is only atomic per writer.",
    ),
    (
        "src/core/download.rs",
        "A streaming body cannot use `durable_write`, which takes the bytes as a buffer and so \
         would undo II.55's chunked write. It streams to a sibling, flushes and `sync_all`s \
         there, and the commit is a rename of a file that is already durable — the same order, \
         with the caller's checksum and chmod in between because a rename has to be the last \
         thing that happens.",
    ),
];

/// **Judged per file, because the offence is a sequence and not a line.** Writing content into
/// a temporary file and renaming it over a target is the thing that has to be one implementation;
/// a bare `rename` is a *move*, and a `NamedTempFile` nobody persists is a temporary file being
/// used as one.
///
/// Getting that wrong the other way is not hypothetical: the first version of this scan flagged
/// `app/sandbox.rs` (writes a `.wsb` config to a temp file and hands the sandbox its path),
/// `backends/link.rs` (restores a user's backup over their file) and `core/journal.rs` (sets a
/// corrupt WAL aside). None of the three writes a file atomically; all three would have been
/// "fixed" into using a writer that does not fit them.
///
/// **The streaming spelling is in here because the sequence was missed in exactly that shape.**
/// `backends/appimage.rs` downloaded to `<artifact>.shall-part`, chmodded it, and renamed it over
/// the live binary — the whole offence, hand-rolled, in the one backend whose artifact is on
/// `PATH` — and this scan did not see it, because it only knew `fs::write` and `NamedTempFile`
/// and a download is neither. Two spellings of the same sequence, so both are looked for.
fn renames_into_place(source: &str) -> bool {
    let code: String = source
        .lines()
        .map(|l| l.split("//").next().unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n");
    // `.persist(` with a dot is `tempfile`'s rename-over-the-target; `file::persist(` — the
    // sanctioned front door — has no dot before the name, which is the whole of the distinction.
    let persists_a_temp_file = code.contains(".persist(");
    // The hand-rolled spellings: bytes to a path, then a rename onto the real one. `fs::write` is
    // the one-shot form; a `File::create` or `write_all` is the streaming form, which is how a
    // download reaches the disk.
    let renames = code.contains("fs::rename(")
        || code.contains("::rename(")
        || code.contains("fs::rename_all(");
    let writes = code.contains("fs::write(")
        || code.contains("File::create(")
        || code.contains("write_all(");
    persists_a_temp_file || (renames && writes)
}

/// Every `src/` file with a rename-into-place in it, excluding test modules — a test that writes
/// a fixture through a temp file is not a product writer.
fn files_that_rename(root: &Path) -> (BTreeSet<String>, usize) {
    let mut found = BTreeSet::new();
    let mut scanned = 0;
    let mut stack = vec![root.join("src")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let Ok(src) = std::fs::read_to_string(&path) else {
                continue;
            };
            scanned += 1;
            // Everything from the first `#[cfg(test)]` on belongs to the tests.
            let product = src.split("#[cfg(test)]").next().unwrap_or(&src);
            if renames_into_place(product) {
                found.insert(
                    path.strip_prefix(root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    (found, scanned)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn only_the_sanctioned_writers_rename_a_file_into_place() {
    let (found, scanned) = files_that_rename(&repo_root());

    Ledger::of("a rename into place of their own", "MAY_RENAME")
        .pairs(MAY_RENAME)
        .scanning_at_least(100)
        .reason_of_at_least(60)
        .remedy(
            "Use `utils::file::persist` (the config repo, preview-aware) or \
             `CommandExecutor::write_atomic`/`write_secret` (the machine, VFS-aware). Both go \
             through `durable_write`, which flushes and fsyncs before the rename — the three \
             steps two of the four hand-rolled copies were missing.",
        )
        .audit(scanned, &found);
}

/// **The oracle.** The predicate is driven over planted lines, so a scan that has stopped
/// matching cannot pass by finding nothing.
#[test]
fn the_scan_can_actually_fail() {
    // The two spellings of the offence.
    assert!(renames_into_place(
        "let t = NamedTempFile::new_in(dir)?;\nt.persist(path)?;"
    ));
    assert!(renames_into_place(
        "std::fs::write(&tmp, json)?;\nstd::fs::rename(&tmp, &path)?;"
    ));
    // **And the spelling the scan missed for a whole release of `appimage:`** — a streamed body
    // into a path, then that path renamed over the live artifact. Planted from the code it was
    // written from, so a future edit to the scan cannot quietly narrow the shape back to the one
    // that did not see it.
    assert!(
        renames_into_place(
            "let mut f = tokio::fs::File::create(&part_path).await?;\n\
             f.write_all(&chunk).await?;\n\
             tokio::fs::rename(&part_path, &dest_path).await?;"
        ),
        "the scan cannot see a streaming write followed by a rename, which is what appimage did"
    );
    assert!(renames_into_place(
        "file.write_all(b\"x\")?;\nstd::fs::rename(tmp, dest)?;"
    ));

    // The three real files that are NOT the offence, in the shape they actually have.
    assert!(
        !renames_into_place("let mut tmp = NamedTempFile::new()?;\ncmd.arg(tmp.path());"),
        "a temp file used as a temp file is not an atomic write (app/sandbox.rs)"
    );
    assert!(
        !renames_into_place("tokio::fs::rename(&backup, path).await?;"),
        "restoring a backup is a move (backends/link.rs)"
    );
    assert!(
        !renames_into_place("let moved = std::fs::rename(&self.path, &backup).is_ok();"),
        "setting a corrupt file aside is a move (core/journal.rs)"
    );
    assert!(
        !renames_into_place("crate::utils::file::persist(&p, &body)?;"),
        "the sanctioned front door has no dot before `persist`"
    );
    assert!(!renames_into_place("fs::write(&path, body)?;"));
    assert!(!renames_into_place(
        "// NamedTempFile::new_in(d).persist(p) is how the durable write works"
    ));
    assert!(!renames_into_place(""));

    // And the walk finds a planted file. Driven over a temp tree so the assertion is about the
    // walk rather than about the repo it happens to be pointed at.
    let tmp = tempfile::tempdir().expect("a temp dir");
    let src = tmp.path().join("src").join("deep");
    std::fs::create_dir_all(&src).expect("the tree");
    std::fs::write(
        src.join("offender.rs"),
        "fn save() {\n    let t = NamedTempFile::new_in(d)?;\n    t.persist(p)?;\n}\n",
    )
    .expect("the offender");
    std::fs::write(
        src.join("innocent.rs"),
        "fn save() {\n    crate::utils::file::persist(&p, &body)?;\n}\n",
    )
    .expect("the innocent");
    std::fs::write(
        src.join("only_in_tests.rs"),
        "fn save() {}\n#[cfg(test)]\nmod t {\n    fn f() { NamedTempFile::new_in(d).persist(p); }\n}\n",
    )
    .expect("the test-only file");

    let (found, scanned) = files_that_rename(tmp.path());
    assert_eq!(scanned, 3);
    assert_eq!(
        found.into_iter().collect::<Vec<_>>(),
        vec!["src/deep/offender.rs".to_string()],
        "the walk missed the planted offender, or caught one of the two controls"
    );
}
