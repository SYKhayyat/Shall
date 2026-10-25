use crate::core::{Error, Result};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use tempfile::NamedTempFile;

/// Write a file Shall owns — `active`, `preferences.toml`, a manifest, a lock under `locks/`,
/// the WAL, its own state registry — atomically, and **the one place `--dry-run` stops bytes
/// from reaching a disk**.
///
/// Returns `true` when the bytes were written and `false` when the run is a preview, so a
/// caller can phrase its own message either way without asking the flag a second time.
///
/// It says what it would have done, at a level the default filter shows: the five verbs this
/// exists for did not merely act during a preview, they acted *silently*, and of the two that
/// is the worse half — `--dry-run activate Work` switched the profile and printed nothing at
/// all.
///
/// **There were two of these.** A preview-aware `write_config` and a permissive `atomic_write`,
/// and the second is the one the `save()` methods reached for: `--dry-run adopt` recorded 112
/// packages as managed while the manifest that declares them was correctly not written, leaving
/// the machine in the one state the model reads as *the user deleted every line*. A writer that
/// honours the flag is no protection while a writer that ignores it sits beside it, so there is
/// now one **preview policy for the config repo**.
///
/// **That sentence used to say "so there is now one", full stop, and it was wrong about the
/// thing it sounded like it was claiming.** There are two preview policies and there always
/// were: this one, which prints *would write …* and stops, and the executor's, which diverts the
/// bytes into a dry-run VFS so a later read in the same run sees them. Both are correct and they
/// answer different questions — a manifest a preview must not touch, versus a file the previewed
/// commands would go on to read. What was *not* correct is that each carried its own copy of the
/// rename-into-place dance, and two of the three copies had no `fsync` in them. The durability
/// is one function now ([`durable_write`]); the preview policies remain two, deliberately, and
/// `a_writer_that_reaches_the_disk_goes_through_one_tests` enumerates them.
pub fn persist(path: &Path, content: &str) -> Result<bool> {
    if crate::core::dry_run::active() {
        crate::would_warn!("would write {}", path.display());
        return Ok(false);
    }
    atomic_write(path, content)?;
    Ok(true)
}

/// Add one line to the end of a file, durably. Returns whether the bytes reached the disk.
///
/// For append-only logs, where rewriting the whole file to record one more event is O(n²) in
/// the number of events. Same preview policy as [`persist`]: a run that performs nothing writes
/// nothing.
///
/// A crash partway through leaves a truncated final line rather than a corrupt file, which is
/// the property that makes the format worth having — the reader drops an unparseable tail and
/// keeps everything before it.
/// [`persist`], from an `async fn`, without parking a runtime worker.
///
/// `persist` ends in `sync_all` — a physical flush — so calling it straight from an `async fn`
/// parks the worker on the disk for as long as the disk takes (II.52). Most of the callers that
/// did that were writing a whole registry while holding the lock that everything else wants;
/// the rest are one-shot writes at the end of a command, where the cost is small and the rule
/// is still the rule. Having one door means there are no exceptions to remember, which is what
/// `a_blocking_wait_is_off_the_runtime` enforces.
pub async fn persist_off_the_runtime(path: &Path, content: &str) -> Result<bool> {
    let path = path.to_path_buf();
    let content = content.to_string();
    crate::core::off_the_runtime(move || persist(&path, &content)).await?
}

pub fn append_line(path: &Path, line: &str) -> Result<bool> {
    append_lines(path, std::slice::from_ref(&line))
}

/// Add several lines to the end of a file, durably, in **one** flush.
///
/// **The guarantee is the same one `append_line` gives, and the cost is not.** `sync_data` is a
/// physical flush; a caller opening a whole wave's worth of WAL entries paid one per entry, so
/// a 298-package config spent ~298 flushes on the critical path — each of them, because the
/// hottest fan-outs are multiplexed onto a single task, stalling the whole wave rather than one
/// member of it. All the lines are written and then flushed once, so every one of them is on
/// disk before this returns, which is precisely what the per-entry version promised.
///
/// A crash partway through leaves a truncated final line rather than a corrupt file, which is
/// the property that makes the format worth having — the reader drops an unparseable tail and
/// keeps everything before it.
pub fn append_lines(path: &Path, lines: &[&str]) -> Result<bool> {
    if crate::core::dry_run::active() {
        crate::would_warn!("would append to {}", path.display());
        return Ok(false);
    }
    if lines.is_empty() {
        return Ok(false);
    }
    if let Some(dir) = path.parent() {
        ensure_dir(dir)?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(Error::from)?;
    // A file whose last line was written without one — a `.gitignore` a user edited by hand,
    // a log a crash truncated — would otherwise have this appended onto the end of it.
    let mut body = String::new();
    if ends_mid_line(path) {
        body.push('\n');
    }
    for line in lines {
        body.push_str(line);
        body.push('\n');
    }
    file.write_all(body.as_bytes()).map_err(Error::from)?;
    file.sync_data().map_err(Error::from)?;
    Ok(true)
}

/// Whether `path` has content that does not end in a newline. A missing or empty file does not.
///
/// **One byte is wanted, so one byte is read**: the WAL this serves grows with a machine's
/// whole history, and reading all of it — once or twice per wave, every run — was O(events²)
/// across that history. Open and seek to the last byte instead; the file cannot shrink under
/// an open handle's seek in any way that makes the answer wrong.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn ends_mid_line(path: &Path) -> bool {
    use std::io::{Seek, SeekFrom};
    let Ok(mut f) = fs::File::open(path) else {
        return false;
    };
    if f.seek(SeekFrom::End(-1)).is_err() {
        // Includes the empty file: nothing to seek to, and empty does not end mid-line.
        return false;
    }
    !matches!(std::io::Read::bytes(&mut f).next(), Some(Ok(b'\n')))
}

/// The bytes, atomically, with no policy.
///
/// Private on purpose — [`persist`] is the way in, so a new writer cannot reach the disk
/// during a preview by picking the shorter name. **One caller exists outside this module by
/// explicit sanction** (the MAY_RENAME ledger): `model/edit.rs`'s two writers, which carry
/// their own preview policy (`Writes::Planned`) and were hand-rolling their own rename dance
/// until the audit caught it. Everything else goes through [`persist`].
pub(crate) fn atomic_write(path: &Path, content: &str) -> Result<()> {
    durable_write(path, content, |_| Ok(()))
}

/// **The one durable write.** Bytes into a temporary file beside the destination, flushed,
/// `prepare`d, fsynced, then renamed over the target.
///
/// **Every step is load-bearing and three of them were missing from two of the three callers.**
/// `rename` is atomic against a concurrent *reader* — nobody ever sees a half-written file — and
/// says nothing at all about power loss: a rename can reach the disk before the bytes it points
/// at do, which leaves a file of the right name and zero length. `CommandExecutor::write_atomic`
/// omitted both `flush` and `sync_all` and is what writes a systemd unit and a `link:` target,
/// while `registry.json` and the WAL went through here and survived. That is the worst possible
/// division: a crash keeps Shall's record of what it did and loses what it did.
///
/// `prepare` runs on the temporary file **after** the bytes and **before** the rename, which is
/// the only window in which a mode change is not a window. A `chmod` after the rename means the
/// target path holds world-readable plaintext for however long that takes, and for a secret
/// "however short" is not an argument (T5).
///
/// `pub(crate)` rather than private because the executor's two writers are the other legitimate
/// front door — they answer to the dry-run VFS instead of to [`persist`]'s preview check — and a
/// second copy of this dance is exactly what they were.
pub(crate) fn durable_write(
    path: &Path,
    content: &str,
    prepare: impl FnOnce(&Path) -> Result<()>,
) -> Result<()> {
    let dir = path.parent().ok_or_else(|| {
        let err = std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Target path has no parent directory",
        );
        Error::Io(err.to_string())
    })?;

    ensure_dir(dir)?;

    let mut temp_file = NamedTempFile::new_in(dir).map_err(Error::from)?;
    temp_file
        .write_all(content.as_bytes())
        .map_err(Error::from)?;
    temp_file.flush().map_err(Error::from)?;
    prepare(temp_file.path())?;
    temp_file.as_file().sync_all().map_err(Error::from)?;
    temp_file.persist(path).map_err(Error::from)?;

    // The rename is durable only once the directory entry naming it is. Without this, the
    // first-ever creation of a journal or state file can vanish whole in a power cut: the file
    // was fsynced, but nothing forced the directory that says it exists.
    #[cfg(unix)]
    {
        if let Ok(d) = std::fs::File::open(dir) {
            let _ = d.sync_all();
        }
    }

    Ok(())
}

/// Create `path` and its parents, and say which directory failed if it does.
///
/// **The error is the reason this exists.** `create_dir_all(p)?` on a read-only mount or a
/// path under a directory the user cannot write produces `Access is denied. (os error 5)` and
/// nothing else — no path, in a program that has just touched a dozen of them. It is also
/// why the old body's `if !path.exists()` guard is gone: `create_dir_all` is already a no-op on
/// an existing directory, so the check bought nothing and lost the race between the two calls.
pub fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|e| dir_error(path, &e))
}

/// [`ensure_dir`] for a caller already inside an async context, so the two cannot report the
/// same failure two ways.
pub async fn ensure_dir_async(path: &Path) -> Result<()> {
    tokio::fs::create_dir_all(path)
        .await
        .map_err(|e| dir_error(path, &e))
}

fn dir_error(path: &Path, e: &std::io::Error) -> Error {
    Error::Io(format!("could not create {}: {e}", path.display()))
}

/// The lines of a list file, with blanks and `#` comments dropped.
///
/// A missing file is an empty list, not an error: every caller reads an optional file that the
/// user has simply not written yet.
pub fn read_lines_filtered(path: &Path) -> Result<Vec<String>> {
    if !path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(path).map_err(Error::from)?;
    Ok(filtered_lines(&content))
}

/// The filter itself, for a caller that already holds the text.
pub fn filtered_lines(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect()
}

/// Delete `path` whether it is a file or a directory, and treat "it was not there" as done.
///
/// **`symlink_metadata`, and the `is_symlink` clause, are the whole function.** `path.is_dir()`
/// follows the link, so a symlink pointing at a directory answered yes and got `remove_dir_all`
/// — which deletes the *target's* contents and leaves the link. `link:` removes exactly this
/// kind of path for a living, which is why its two copies of this branch already had the
/// clause and the one here did not.
pub fn force_remove(path: &Path) -> Result<()> {
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        // Not `if path.exists()` first: between the check and the delete the path can go, and
        // a removal that fails because the thing is already gone has done its job.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(removal_error(path, &e)),
    };
    match remove_by_kind(path, &meta) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(removal_error(path, &e)),
    }
}

/// Delete what `meta` describes, never what it points at.
///
/// **Windows needs the symlink arm split out**: `remove_file` cannot delete a directory symlink
/// (`Access is denied`) and `remove_dir` cannot delete a file one, and `symlink_metadata`
/// reports both as neither file nor directory. `try the file form, then the directory form` is
/// the whole rule, and it covers a dangling link — whose target kind cannot be asked for at all.
fn remove_by_kind(path: &Path, meta: &fs::Metadata) -> std::io::Result<()> {
    if meta.is_symlink() {
        // Both forms tried, and when both fail **both errors reported**: the file form's
        // error used to be swallowed for whatever the directory form said, and on Windows
        // that second message is routinely the vaguer of the two.
        return match fs::remove_file(path) {
            ok @ Ok(()) => ok,
            Err(file_err) => match fs::remove_dir(path) {
                Ok(()) => Ok(()),
                Err(dir_err) => Err(std::io::Error::new(
                    dir_err.kind(),
                    format!("{file_err} (directory form: {dir_err})"),
                )),
            },
        };
    }
    if meta.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn removal_error(path: &Path, e: &std::io::Error) -> Error {
    Error::Io(format!("could not remove {}: {e}", path.display()))
}

/// The path inside `bin_dir` that an `@bin=` value names, refused if it names anywhere else
/// (SEC1).
///
/// `@bin=` is user text, and text a user pastes from somewhere else. Joined blindly,
/// `@bin=../../.bashrc` resolves to a file in the home directory, and the deploy that follows
/// replaces it with a symlink to a freshly downloaded, attacker-chosen file — one pasted line,
/// code on the next shell start. **The traversal is in the destination, so HTTPS and a matching
/// checksum do not touch it**: a fully verified download lands wherever `@bin` points.
///
/// A bin name is a filename. Anything with a separator in it, anywhere, is refused rather than
/// normalised, because "what does `a/../b` mean" is a question with a different answer on every
/// filesystem and none of them is worth being wrong about.
///
/// `confined` is the `[guard] confine_bin` key: off restores the unchecked join. The opening is
/// the user's to make, and it is the whole file's worth of blast radius.
/// Every suffix that comes off a downloaded file's name.
///
/// **The archive half is `Format`'s own table**, not a fourth hand-written copy of it — this
/// list carried `.tar.zst` for as long as `extract_archive` could not open one, which is how
/// four lists of one fact stay wrong in four different ways. What is spelled out here is only
/// what `Format` does not know about: the bare codec tails, which name a compressed file rather
/// than an artifact Shall would ever select, and `.7z`, which nothing opens and everything
/// should still strip.
///
/// **Sorted longest-first, rather than written that way.** The lookup below takes the first
/// match, so `.gz` sitting above `.tar.gz` would silently cut `ripgrep.tar.gz` down to
/// `ripgrep.tar`. That was a hand-maintained ordering with a comment asking future editors to
/// preserve it; it is a property of the list now.
static ARCHIVE_SUFFIXES: once_cell::sync::Lazy<Vec<&'static str>> =
    once_cell::sync::Lazy::new(|| {
        use crate::backends::artifact::format::Format;
        let mut all: Vec<&'static str> = Format::ALL
            .into_iter()
            .filter(|f| f.is_archive())
            .flat_map(|f| f.suffixes().iter().copied())
            .chain([".gz", ".bz2", ".xz", ".zst", ".7z", ".exe", ".appimage"])
            .collect();
        all.sort_by_key(|s| std::cmp::Reverse(s.len()));
        all
    });

/// The name a downloaded file installs under.
///
/// Only known suffixes come off, and repeatedly: cutting at the first `.` turned
/// `ripgrep-14.1.0-x86_64.tar.gz` into `ripgrep-14`, and that misnamed binary is what
/// landed on PATH.
pub fn strip_archive_suffixes(filename: &str) -> &str {
    let mut name = filename;
    loop {
        let lower = name.to_ascii_lowercase();
        match ARCHIVE_SUFFIXES.iter().find(|s| lower.ends_with(*s)) {
            Some(suffix) => name = &name[..name.len() - suffix.len()],
            None => return name,
        }
    }
}

/// Why `name` is not a plain file name that stays inside a directory, or `None` if it is one.
///
/// **One `Component::Normal` and nothing else.** Spelling the refusals out one form at a time
/// is what let `C:evil` through `@bin=`: it holds no separator, it is not `..`, and
/// `is_absolute()` is false for a drive letter with no root — yet `join` discards the base
/// directory for it exactly as it does for an absolute path. Whatever the platform counts as
/// more than a bare file name, `components()` already knows.
///
/// **And what one platform counts, the other must count too.** `components()` answers for the
/// platform it is compiled on: on Unix a backslash is an ordinary filename character, so
/// `..\..\x` is one `Normal` component and this returned `None` for it. That is not an escape
/// on the machine that parsed it — the file lands inside the bin directory, backslashes and
/// all — but a manifest is a file that travels, and the same line is a traversal on Windows.
/// A rule that means one thing per platform is a rule the user cannot check by reading it, so
/// both separators are refused everywhere and the answer is a property of the text.
///
/// Shared with [`url_filename`] because the two questions are one question: both take text
/// from outside and turn it into a name to join onto a directory Shall owns.
fn not_a_bare_file_name(name: &str) -> Option<&'static str> {
    if name.is_empty() {
        return Some("it is empty");
    }
    if name.contains('/') || name.contains('\\') {
        return Some("it contains a path separator");
    }
    let mut parts = Path::new(name).components();
    match (parts.next(), parts.next()) {
        (Some(Component::Normal(_)), None) => {}
        (Some(Component::Prefix(_)), _) => {
            return Some("it names a drive, not a file inside the directory")
        }
        (Some(Component::RootDir), _) => return Some("it is an absolute path"),
        (Some(Component::CurDir), _) | (Some(Component::ParentDir), _) => {
            return Some("it is a directory, not a name")
        }
        (Some(_), Some(_)) => return Some("it contains a path separator"),
        (None, _) => return Some("it is not a file name"),
    }
    // The write would go to a console or a printer port rather than to a file, and on Windows
    // that holds however the name is spelled and whatever extension follows it.
    #[cfg(windows)]
    if is_reserved_device_name(name) {
        return Some("it is a reserved device name, so the write reaches a device, not a file");
    }
    None
}

/// The file name a URL installs under: the last path segment, and nothing else.
///
/// **The last `/`-separated chunk of the raw URL is not a file name.** It carries the query
/// string and the fragment, which CDN-signed and redirect-generated asset URLs routinely have:
/// `https://host/tool.AppImage?token=abc` became a file called `tool.AppImage?token=abc`, which
/// on Windows is an illegal name so `File::create` failed with an I/O error naming nothing, and
/// on Unix succeeded and put the token in the name — and then in the `@bin` link name too. A
/// URL ending in `/` gave `""`, so `join` returned the install directory itself.
///
/// Percent-encoding is deliberately left alone. Decoding is how a `%2F` becomes a separator,
/// and a separator is the one thing this must not produce.
pub fn url_filename(url: &str) -> Result<String> {
    let refuse = |why: &str| {
        Err(Error::Validation(format!(
            "cannot tell what file `{}` names: {}. A download URL has to end in a file name \
             — that name is what lands in the install directory and what goes on PATH.",
            url, why
        )))
    };
    let parsed = match reqwest::Url::parse(url) {
        Ok(parsed) => parsed,
        Err(e) => return refuse(&format!("it is not a URL ({})", e)),
    };
    let name = parsed
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .unwrap_or("")
        .to_string();
    match not_a_bare_file_name(&name) {
        Some(why) => refuse(why),
        None => Ok(name),
    }
}

/// Windows reserves these names at every directory, with or without an extension: opening
/// `NUL.exe` opens the null device. A confined `@bin=` must not resolve to one, or the deploy
/// writes into a device and reports success having produced no file.
#[cfg(windows)]
fn is_reserved_device_name(name: &str) -> bool {
    const RESERVED: [&str; 22] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let stem = name.split('.').next().unwrap_or(name).trim_end_matches(' ');
    RESERVED.iter().any(|r| stem.eq_ignore_ascii_case(r))
}

pub fn bin_destination(bin_dir: &Path, name: &str, confined: bool) -> Result<PathBuf> {
    let refuse = |why: &str| {
        Err(Error::Validation(format!(
            "refusing `@bin={}`: {}. It names a file inside {}, and nothing else — a value \
             that escapes it would put a downloaded file wherever it pointed. Set \
             `[guard] confine_bin = false` if you really mean it.",
            name,
            why,
            bin_dir.display()
        )))
    };
    if confined {
        if let Some(why) = not_a_bare_file_name(name) {
            return refuse(why);
        }
    }

    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut dest = bin_dir.join(name);
    // The recorded path has to be the one that was written, extension and all, or the removal
    // path looks for a file that was never there.
    #[cfg(windows)]
    if dest.extension().is_none() {
        dest.set_extension("exe");
    }
    Ok(dest)
}

/// Put a downloaded artifact's executable on the user's PATH, refusing to destroy a file
/// Shall did not deploy.
///
/// `dest` must come from [`bin_destination`], which is what keeps an `@bin=` value from
/// naming a file outside the bin directory (SEC1).
///
/// `~/.local/bin` is shared with the user and with every other tool that installs there, so
/// deploying by name alone means a package called `fd` silently replaces whatever `fd` the
/// user already had. `ShimManager` has always refused that; the download backends each
/// hand-rolled a symlink that did not, so the same directory had opposite answers depending on
/// which backend got there first.
///
/// A destination counts as Shall's when it is absent, when it is a symlink pointing inside
/// `owned_root` (the backend's own install directory), or when it is the exact path this
/// backend recorded deploying last time — which is what identifies a copy, since a copy
/// carries no pointer home.
pub async fn deploy_executable(
    src: &Path,
    dest: &Path,
    owned_root: &Path,
    recorded: Option<&str>,
) -> Result<()> {
    ensure_deployable(dest, owned_root, recorded).await?;

    // **The check is where the write is, per `core::dry_run`.** The download backends reach
    // their filesystem writes outside the `persist`/`ensure_dir` funnel that carries this rule
    // for everything else, so the only thing keeping a preview from putting a file on PATH was
    // each verb returning before `install()` — which is the per-verb habit that module exists
    // to delete. A verb added tomorrow inherits the rule by calling this.
    if crate::core::dry_run::active() {
        crate::would!("put {} on PATH", dest.display());
        return Ok(());
    }

    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(Error::from)?;
    }
    // The old entry must be gone before the new one is made: symlink and copy both fail onto
    // an existing path, and a dangling symlink reports as absent to `try_exists`.
    if tokio::fs::symlink_metadata(dest).await.is_ok() {
        tokio::fs::remove_file(dest).await.map_err(Error::from)?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = tokio::fs::metadata(src).await.map_err(Error::from)?;
        let mut perms = meta.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(src, perms)
            .await
            .map_err(Error::from)?;
        tokio::fs::symlink(src, dest).await.map_err(Error::from)?;
    }

    #[cfg(windows)]
    {
        // No symlink: it needs a privilege the user may not have, and the copy is what the
        // Windows backends already did.
        tokio::fs::copy(src, dest).await.map_err(Error::from)?;
    }

    Ok(())
}

/// The whole of [`deploy_executable`]'s refusal, asked without any bytes.
///
/// The test is a pure function of the *destination*, so a backend can ask it before it spends
/// the network — and must. Asking only at deploy time cost one `heal` 180 of its 201 seconds
/// fetching two GitHub artifacts it was always going to reject, silently, with no child process
/// to show for it: from outside, indistinguishable from a hang.
pub async fn ensure_deployable(
    dest: &Path,
    owned_root: &Path,
    recorded: Option<&str>,
) -> Result<()> {
    if is_ours(dest, owned_root, recorded).await {
        return Ok(());
    }
    Err(Error::Refused(format!(
        "refusing to deploy `{}`: {} already exists and Shall did not create it. Move or \
         rename that file yourself if you want it managed here.",
        dest.file_name().unwrap_or_default().to_string_lossy(),
        dest.display()
    )))
}

async fn is_ours(dest: &Path, owned_root: &Path, recorded: Option<&str>) -> bool {
    let Ok(meta) = tokio::fs::symlink_metadata(dest).await else {
        return true; // absent
    };
    if recorded.is_some_and(|r| Path::new(r) == dest) {
        return true;
    }
    if meta.file_type().is_symlink() {
        if let Ok(target) = tokio::fs::read_link(dest).await {
            return target.starts_with(owned_root);
        }
    }
    false
}

/// Delete a file or directory a backend deployed, reporting whether it is actually gone.
///
/// An already-absent path counts as removed: the caller's goal is "not on disk", and
/// `NotFound` means that goal is met. Any other error means the file is still there and
/// still on the user's PATH, which the caller must not record as a clean removal.
pub async fn remove_deployed_path(path: impl AsRef<Path>) -> std::result::Result<(), String> {
    let path = path.as_ref();
    let meta = match tokio::fs::symlink_metadata(path).await {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(format!("{}: {}", path.display(), e)),
    };
    // Blocking, deliberately: this is one `unlink`, and it is the same decision tree as the
    // synchronous remover — two copies of *which* removal call a symlink needs is how one of
    // them ends up with the Windows arm and the other without it.
    let owned = path.to_path_buf();
    match tokio::task::spawn_blocking(move || remove_by_kind(&owned, &meta)).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(Err(e)) => Err(format!("{}: {}", path.display(), e)),
        Err(e) => Err(format!("{}: {}", path.display(), e)),
    }
}

/// Copy `from` over `to`, whatever `to` currently is, and name the path if it cannot.
///
/// Two things `tokio::fs::copy` alone does not do. It cannot open a read-only destination for
/// writing — and a restored config root is full of them, because `bundle` copies the whole
/// root, that root is a git repo, and git writes its objects at 0444 which `copy` carries
/// across. Removing the destination first is what makes an overwrite an overwrite. (Running as
/// root hides this entirely, which is why every container run was green.)
///
/// And its error is `Permission denied (os error 13)` with no path in it, on one of several
/// hundred copies. An I/O error that names no file is one nobody can act on.
pub async fn copy_over(from: &Path, to: &Path) -> Result<()> {
    // Only an existing FILE is removed: a directory in the way is a different fault, and
    // deleting one to make room for a file would turn a mistake into data loss.
    //
    // **And a symlink is removed by the one place that knows which call it needs.** This used to
    // ask `symlink_metadata` and then always reach for `remove_file`. `symlink_metadata` does not
    // follow the link, so a **directory** symlink answered "not a directory" and went to the file
    // form — right on Unix, where a link is a link, and `Access is denied` on Windows, where
    // `remove_file` cannot delete a directory symlink (`PLAN.md` #87). `remove_deployed_path` is
    // the async face of `remove_by_kind`, which tries the file form, then the directory form, and
    // reports both errors when neither works; it is the arm `force_remove` already uses, so the
    // Windows symlink rule now has one home here instead of three.
    //
    // **Not `remove_by_kind` for every path, and the difference is the refusal above.** On a real
    // directory that function calls `remove_dir_all`, so reaching for it unconditionally would turn
    // "a directory in the way is a different fault" into an emptied tree and a successful copy.
    match tokio::fs::symlink_metadata(to).await {
        Ok(meta) if meta.is_symlink() => {
            remove_deployed_path(to)
                .await
                .map_err(|e| Error::Io(format!("could not replace {}: {e}", to.display())))?;
        }
        Ok(meta) if !meta.is_dir() => {
            tokio::fs::remove_file(to)
                .await
                .map_err(|e| Error::Io(format!("could not replace {}: {}", to.display(), e)))?;
        }
        // A real directory, or nothing there at all: both fall through to the copy below, and a
        // directory in the way is refused there — by the filesystem, which cannot open one for
        // writing.
        _ => {}
    }
    tokio::fs::copy(from, to).await.map_err(|e| {
        Error::Io(format!(
            "could not copy {} to {}: {}",
            from.display(),
            to.display(),
            e
        ))
    })?;
    Ok(())
}

/// Move a fully-staged directory into `dest`, keeping the old tree until the new one is in
/// place, and putting it back if the swap itself fails.
///
/// The reinstall paths used to `remove_dir_all(dest)` and only then unpack: one failed archive
/// mid-way left no installation on disk while the state registry still recorded one — a machine
/// Shall believed satisfied over a tool that no longer existed. Same-filesystem renames are
/// atomic; the old tree is dropped only after the new one answers at `dest`.
pub async fn swap_into_place(stage: &Path, dest: &Path) -> Result<()> {
    let retired = dest.with_extension("retired");
    let existed = tokio::fs::try_exists(dest).await.unwrap_or(false);
    if existed {
        let _ = tokio::fs::remove_dir_all(&retired).await;
        tokio::fs::rename(dest, &retired).await.map_err(|e| {
            Error::Io(format!(
                "could not set aside the old {}: {}",
                dest.display(),
                e
            ))
        })?;
    }
    if let Err(e) = tokio::fs::rename(stage, dest).await {
        if existed {
            // Put the old tree back before reporting: a failed install must not be the run
            // that destroyed the working copy.
            let restore = tokio::fs::rename(&retired, dest).await;
            if let Err(re) = restore {
                return Err(Error::Io(format!(
                    "swap of {} failed ({e}) AND the old tree could not be restored: {re}",
                    stage.display()
                )));
            }
        }
        return Err(Error::Io(format!(
            "could not move staged {} into place at {}: {}",
            stage.display(),
            dest.display(),
            e
        )));
    }
    if existed {
        let _ = tokio::fs::remove_dir_all(&retired).await;
    }
    Ok(())
}
