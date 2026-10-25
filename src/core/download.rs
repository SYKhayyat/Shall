//! The rules a remote download obeys before anything it produced reaches your PATH (SEC2).
//!
//! `web:`, `appimage:` and `github:` all do the same three things: fetch a URL, mark the
//! result executable, and put it on `PATH`. That is a code-execution path with the network on
//! the other end of it, so **HTTPS and a checksum are the default and each relaxation is an
//! explicit, separate flag on the line that needs it**:
//!
//! - `@allow_http` — the URL may be `http://`.
//! - `@unverified` — no `@sha256` is required.
//!
//! **They never imply each other.** Allowing plain HTTP for a host that only serves HTTP must
//! not silently also drop the checksum: that combination is precisely the one where the
//! checksum is doing the most work, because anyone on the path can rewrite the response.
//!
//! *Why per-line and not a config key:* a global "require checksums" switch gets turned off
//! once, by the first person who meets a publisher that does not publish hashes, and never
//! gets turned back on — leaving a system that looks protected and is not. A flag on the line
//! has to be written for each spec that needs it, and it stays in the file where the next
//! reader sees it.

use crate::core::{Error, PackageSpec, Result};

/// The ceiling on one downloaded body, in bytes. 2 GiB.
///
/// **Sized to the largest thing anyone legitimately ships this way**, which is an AppImage: the
/// fattest in common use are a few hundred megabytes, so this is roughly eight times the real
/// worst case. It is not a security boundary — a hostile server that stays under it still gets a
/// file written — it is the bound that stops a redirect to something enormous, or a server that
/// never stops sending, from filling the disk while Shall reports progress.
pub const DEFAULT_MAX_DOWNLOAD_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// Seeded from `Config::max_download_bytes`; `0` removes the bound.
static MAX_DOWNLOAD_BYTES: once_cell::sync::OnceCell<u64> = once_cell::sync::OnceCell::new();

/// Set the process-wide download ceiling (called once during startup). Later calls no-op.
pub fn set_max_download_bytes(bytes: u64) {
    let _ = MAX_DOWNLOAD_BYTES.set(bytes);
}

fn max_download_bytes() -> Option<u64> {
    match *MAX_DOWNLOAD_BYTES
        .get()
        .unwrap_or(&DEFAULT_MAX_DOWNLOAD_BYTES)
    {
        0 => None,
        bytes => Some(bytes),
    }
}

/// A response body on disk, whole and synced, **not yet at its destination**.
///
/// The two-phase shape is the whole of the fix: bytes land on a sibling of the destination, so
/// everything that can go wrong — the cap, a dropped connection, a body that stops early, a
/// checksum that does not match — happens while the artifact that is already in place is still
/// in place. Nothing reaches `dest` until the caller has verified the staged bytes and called
/// [`Staged::commit`], and the commit is a rename, which is atomic against a reader.
#[derive(Debug)]
pub struct Staged {
    /// `None` under `--dry-run`, where nothing was written and there is nothing to commit.
    temp: Option<std::path::PathBuf>,
    dest: std::path::PathBuf,
    bytes: u64,
}

impl Staged {
    /// Where the bytes are, for the verification that has to happen before the commit.
    pub fn path(&self) -> &std::path::Path {
        self.temp.as_deref().unwrap_or(&self.dest)
    }

    /// How many bytes arrived, for the summary a caller prints.
    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Whether anything was written at all — `false` only on the `--dry-run` exit.
    ///
    /// Asked before a checksum is verified rather than assumed: the dry-run path has no staged
    /// file, and a caller that verified `path()` there would be reading **the artifact already
    /// on disk** and reporting a mismatch that says nothing about this download.
    pub fn is_staged(&self) -> bool {
        self.temp.is_some()
    }

    /// Put the staged bytes at the destination, atomically.
    ///
    /// Takes `self` because the staged file's lifetime ends here: the rename consumes it, and a
    /// `Staged` that falls out of scope without a commit removes the staged file rather than
    /// leaving a partial one where the next run can find it and treat it as complete.
    pub async fn commit(mut self) -> Result<()> {
        let Some(temp) = self.temp.take() else {
            return Ok(());
        };
        match tokio::fs::rename(&temp, &self.dest).await {
            Ok(()) => Ok(()),
            Err(e) => {
                let _ = tokio::fs::remove_file(&temp).await;
                Err(Error::from(e))
            }
        }
    }
}

impl Drop for Staged {
    fn drop(&mut self) {
        if let Some(temp) = self.temp.take() {
            // A `Drop` cannot await, so this is the one blocking call in the download path: a
            // single unlink of a file this handle owns. It is not on the blocking list in
            // `a_blocking_wait_is_off_the_runtime_tests.rs` because it is neither an fsync nor a
            // walk, and it happens on the error exit where the run is already reporting.
            let _ = std::fs::remove_file(temp);
        }
    }
}

/// Write a response body to `dest`, refusing one that is larger than the ceiling.
///
/// **Two phases, and the split is the point.** The bytes are streamed to a sibling of `dest` and
/// the caller verifies *those* before [`Staged::commit`] moves them into place, so a transfer
/// that dies at 60% cannot leave a truncated file where a working binary was. This function used
/// to write straight onto `dest` and delete it on the way out, which meant a failed download
/// destroyed the previous artifact as thoroughly as it failed to produce a new one.
pub async fn stage_capped(
    response: reqwest::Response,
    dest: &std::path::Path,
    what: &str,
) -> Result<Staged> {
    // **The check is where the write is** (`core::dry_run`). Every verb that can reach a
    // download backend's `install()` returns before it under `--dry-run` today, so this closes
    // nothing that is open — it moves the rule from five verbs remembering it to the one
    // function that creates the file, which is the argument that module makes for itself.
    if crate::core::dry_run::active() {
        crate::would!("download {}", what);
        return Ok(Staged {
            temp: None,
            dest: dest.to_path_buf(),
            bytes: 0,
        });
    }
    stage_capped_to(response, dest, what, max_download_bytes()).await
}

/// The body of [`stage_capped`] with the ceiling passed in.
///
/// Split out so a test can name its own bound: the process-wide one is a `OnceCell` seeded at
/// startup, and a test that set it would decide the value for every other test in the binary.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub async fn stage_capped_to(
    response: reqwest::Response,
    dest: &std::path::Path,
    what: &str,
    cap: Option<u64>,
) -> Result<Staged> {
    // Read from the header rather than `content_length()`: that method answers from the body's
    // size hint, which a streamed response does not have, so it reports `None` for exactly the
    // transfers the check below exists for. Shall's client enables no response decompression, so
    // the declared length is the length of what arrives.
    let declared = response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .or_else(|| response.content_length());
    if let (Some(cap), Some(declared)) = (cap, declared) {
        if declared > cap {
            return Err(refused_size(what, declared, cap));
        }
    }

    // **The staged name is a sibling of the destination, not a tempdir path**, because the commit
    // is a `rename` and a rename across a filesystem is not one — it is a copy, which reintroduces
    // the window this exists to close. Fixed rather than pid-suffixed because two runs cannot
    // share a destination: `shall.lock` is held for the length of a run, so a name that could be
    // contended would only be a way to be wrong later.
    //
    // **A staged file left by a kill is removed on entry, not left to rot.** The only way one
    // survives to the next run is a SIGKILL, and the next attempt of that same artifact is the
    // only thing that can clean it up — so it is cleaned there, where the name is known.
    let temp = part_path(dest);
    let _ = tokio::fs::remove_file(&temp).await;

    // **One cleanup for every way out, and it is the staged file rather than the destination.**
    // A partial artifact left on disk is one a later run can find and treat as complete, and the
    // checksum that would have caught that is the one `@unverified` is allowed to turn off. The
    // `Staged`'s own `Drop` takes the partial with it on every exit; what this arm adds is the
    // sentence, because a download that failed while a working artifact sat underneath it is not
    // a machine with nothing — and a user who is told only "the transfer failed" has no way to
    // know which of the two they are looking at.
    match stream_capped(response, &temp, what, cap, declared).await {
        Ok(bytes) => Ok(Staged {
            temp: Some(temp),
            dest: dest.to_path_buf(),
            bytes,
        }),
        Err(e) => {
            // **The cleanup is here as well as on `Staged`, and the reason is that there is no
            // `Staged` on this path.** The `Drop` belongs to the value the caller would have
            // received, and the caller receives nothing — so a partial staged file would survive
            // every failure exit, which is the bug this shape was built to end. A guard that only
            // exists on success is not a guard.
            let _ = tokio::fs::remove_file(&temp).await;
            if tokio::fs::metadata(dest).await.is_ok() {
                return Err(e.with_note(format!(
                    " The previous {what} is still in place at {} — nothing on this machine was \
                     changed.",
                    dest.display()
                )));
            }
            Err(e)
        }
    }
}

/// Where a body bound for `dest` is streamed before it is committed.
///
/// **One function, because the test that watches for a leftover staged file has to watch for the
/// same name the writer uses** — a test that spelled the name out would pass against a writer that
/// changed it.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn part_path(dest: &std::path::Path) -> std::path::PathBuf {
    dest.with_extension(match dest.extension() {
        Some(existing) => format!("{}.shall-part", existing.to_string_lossy()),
        None => "shall-part".to_string(),
    })
}

/// The streaming half of [`stage_capped_to`], writing to the staged path and nothing else.
///
/// Every exit here is an error exit, and every one of them is cleaned up by the `Staged` the
/// caller builds — the `Drop` removes the staged file, which is why this function needs no
/// cleanup arm of its own and why it may be written straight through.
async fn stream_capped(
    response: reqwest::Response,
    dest: &std::path::Path,
    what: &str,
    cap: Option<u64>,
    declared: Option<u64>,
) -> Result<u64> {
    use futures::StreamExt;
    use tokio::io::AsyncWriteExt;

    let mut file = tokio::fs::File::create(dest).await.map_err(Error::from)?;
    let mut written: u64 = 0;
    let mut body = response.bytes_stream();
    while let Some(chunk) = body.next().await {
        let chunk = chunk.map_err(Error::from)?;
        written += chunk.len() as u64;
        if let Some(cap) = cap {
            if written > cap {
                return Err(refused_size(what, written, cap));
            }
        }
        file.write_all(&chunk).await.map_err(Error::from)?;
    }
    file.flush().await.map_err(Error::from)?;
    // The bytes become an executable later in this same install; flush moves them to the OS,
    // sync_all is what survives the power cut. The one artifact path that needed saying so.
    file.sync_all().await.map_err(Error::from)?;
    // **A body that stops early is not a body that ended.** Nothing else notices: the stream
    // reports no error, and the hash that would have caught it is the one `@unverified` turns
    // off and the one VIII.2 exempts every `github:` line from. When the server said how many
    // bytes were coming, the count is the check.
    if let Some(declared) = declared {
        if written != declared {
            return Err(Error::command_failed(format!(
                "{} ended after {} of the {} the server said it would send — the transfer was \
                 cut short, and what arrived is not the file.",
                what,
                human_bytes(written),
                human_bytes(declared)
            )));
        }
    }
    Ok(written)
}

/// Permanent: the same URL answers with the same size next time, so a retry spends the transfer
/// again to reach the same refusal.
fn refused_size(what: &str, size: u64, cap: u64) -> Error {
    Error::command_failed_permanently(format!(
        "{} is {} and the ceiling is {} — refused before it filled the disk. Raise \
         `max_download_bytes`, or set it to 0 to remove the bound.",
        what,
        human_bytes(size),
        human_bytes(cap)
    ))
}

/// Bytes as a person reads them. A refusal that says `2147483648` makes the reader do the
/// arithmetic before they can decide whether the number is wrong.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn human_bytes(n: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
    let mut value = n as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", n, UNITS[0])
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}

/// Whether a bare flag is set on a spec. The grammar stores a bare `@flag` as `"true"`.
fn flag(spec: &PackageSpec, name: &str) -> bool {
    spec.options.one(name).is_some_and(|v| v == "true")
}

pub fn allows_http(spec: &PackageSpec) -> bool {
    flag(spec, "allow_http")
}

pub fn is_unverified(spec: &PackageSpec) -> bool {
    flag(spec, "unverified")
}

/// `@system=true` — this line may write into an environment the OS owns (`Q49`).
///
/// Here beside `is_unverified` because it is the same shape of thing: a per-line opt-in to a
/// refusal that exists for a good reason, read once so no caller re-derives it from the option
/// map. It is not about downloads; it is about the flag readers this module already owns.
pub fn is_system(spec: &PackageSpec) -> bool {
    flag(spec, "system")
}

/// Refuse a URL that is not `https://`, unless this spec opted out.
///
/// Applied to **every URL actually fetched, not only the one that was typed**: reqwest follows
/// up to ten redirects, so an `https://` seed can be bounced to `http://` and the check on the
/// typed string would pass while the bytes arrive in clear.
pub fn check_scheme(url: &str, allow_http: bool, what: &str) -> Result<()> {
    if url.starts_with("https://") || allow_http {
        return Ok(());
    }
    Err(Error::Refused(format!(
        "refusing to download {} over plain HTTP: {}\n  \
         The file is made executable and put on your PATH, so anyone between you and that \
         host chooses what runs. Use `https://`, or add `@allow_http` to the line if the \
         publisher genuinely offers nothing else.",
        what, url
    )))
}

/// Refuse a download that carries no `@sha256`, unless this spec opted out.
///
/// **`github:` is exempt, and that is a ruling, not an omission** (owner, 2026-07-21). One
/// GitHub release ships a `.deb`, an `.rpm` and a tarball, so VIII.2 makes a hand-written
/// `@sha256` legal there only when the line pins exactly one format — requiring one would
/// force `@formats=` onto every github line, or push everyone to write `@unverified`, which
/// turns the flag into noise instead of a decision. github's integrity is `locks/github.toml`
/// instead: the hash of what was downloaded is recorded, and the same release arriving with
/// different bytes later is refused. The HTTPS half still applies to it, on every redirect
/// hop.
pub fn check_checksum_declared(spec: &PackageSpec) -> Result<()> {
    if spec.options.contains("sha256") || is_unverified(spec) {
        return Ok(());
    }
    Err(Error::Refused(format!(
        "refusing to install `{}` unverified: no `@sha256` on the line.\n  \
         The downloaded file is made executable and put on your PATH. Add `@sha256=<hash>`, \
         or `@unverified` to say you accept whatever the host serves. `@allow_http` does not \
         cover this — HTTP and no-checksum are separate decisions.",
        spec.name
    )))
}

/// A client whose redirect policy enforces the scheme on every hop.
///
/// The binding requirement is that the *final* download is HTTPS; checking each hop is the
/// cheapest correct form and also catches a downgrade in the middle of a chain that ends back
/// on HTTPS.
/// A download carries no whole-request timeout: a release asset can legitimately take an hour,
/// and a bound sized for an API call turns a slow link into a corrupt install.
pub fn client(allow_http: bool, user_agent: &str) -> Result<reqwest::Client> {
    crate::core::http::client(user_agent, allow_http, 0)
}
