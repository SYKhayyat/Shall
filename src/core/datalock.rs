//! One writer at a time on the data directory (II.8).
//!
//! Shall is not the only thing that starts Shall: the package-manager hooks it installs
//! (`DPkg::Post-Invoke` and its siblings) spawn a reconcile on every ordinary `apt install`,
//! typed by someone who has never heard of this tool. `registry.json`, the journal and the
//! `locks/` ledgers are written whole, and two whole writes are last-one-wins â€” the entry
//! that loses is a managed package nothing declares, which is drift, and drift is removed.
//!
//! The lock covers the directory rather than one file: those files must agree with each
//! other, and a lock over one of a set that must agree is the same as no lock.

use crate::core::{Error, Result};
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn is_lock_contended(error: &std::io::Error) -> bool {
    if matches!(
        error.kind(),
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::AlreadyExists
    ) {
        return true;
    }
    #[cfg(unix)]
    {
        error.raw_os_error() == Some(libc::EWOULDBLOCK)
            || error.raw_os_error() == Some(libc::EAGAIN)
    }
    #[cfg(windows)]
    {
        // LockFileEx/LockFile on Windows reports these as Win32 errors instead of mapping
        // them to std::io::ErrorKind::WouldBlock: ERROR_SHARING_VIOLATION (32) and
        // ERROR_LOCK_VIOLATION (33).
        matches!(error.raw_os_error(), Some(32 | 33))
    }
    #[cfg(not(any(unix, windows)))]
    {
        false
    }
}

/// How long a waiting run gives the holder before it says so instead.
///
/// 120s: long enough to outlast the longest wait a holder can legitimately make before it
/// starts doing work â€” the rate-limit ceiling, 30s by default â€” with room for the install it
/// then performs. It is not meant to outlast a whole sync: past this point the honest answer is
/// that someone else is writing, not a longer silence (S27).
pub const WAIT_SECS: u64 = 120;

/// How many `DataLock`s this process is holding.
///
/// **`flock` is per open file description, not per process**, so a second handle opened in a
/// process that already holds the lock does not re-enter â€” it waits for itself, for ever. Every
/// door that takes the lock counts here, and every door that might take it asks here first.
static HELD: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Whether this process is inside the lock right now.
///
/// The question is dynamic and has to be: `LockScope::Deferred` takes the lock at each mutating
/// action and releases it in between, so no value carried around by the type system can say
/// whether it is held at the moment somebody writes.
pub fn held() -> bool {
    HELD.load(std::sync::atomic::Ordering::Acquire) > 0
}

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so a private item is invisible there.
/// The file that counts writers, so a reader can tell whether one moved underneath it.
pub const GENERATION_FILE: &str = "shall.gen";

/// What a reader saw of the writers, at one instant.
///
/// Two observations that compare equal, with no writer holding the lock at either, mean no
/// writer committed anything in between â€” which is what makes a multi-file read one moment
/// rather than several. See [`crate::core::stable`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Generation {
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    /// Bumped once by every writer that finishes. `None` when the file could not be read or
    /// did not parse â€” see [`observe`].
    pub count: Option<u64>,
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    /// Whether somebody held the lock as this was taken. A reader that saw a writer cannot
    /// conclude anything from two equal counts: the writer may not have released yet.
    pub writer_active: bool,
}

/// Read the writer generation. Two small reads of tiny files, and no lock of any kind â€” a
/// reader must never wait on a writer, which is the whole reason this exists.
pub fn observe(data_dir: &Path) -> Generation {
    // **An unreadable counter is `None`, not `0`.** `0` is a *lower* number than any real
    // generation, so two observations straddling a crashed writer compared equal and
    // `spans_one_moment` said yes to a read that spanned two. A file that has never been
    // written is a genuine `0` â€” nothing has committed yet â€” and that case is kept apart from
    // the torn one on purpose.
    let count = match std::fs::read_to_string(data_dir.join(GENERATION_FILE)) {
        Ok(s) => s.trim().parse::<u64>().ok(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Some(0),
        Err(_) => None,
    };
    Generation {
        count,
        writer_active: data_dir.join("shall.lock.owner").exists(),
    }
}

impl Generation {
    /// Whether a read that spanned these two observations saw one moment.
    pub fn spans_one_moment(self, later: Self) -> bool {
        // `None == None` is true for `Option`, and it must not be an answer here: two
        // unreadable counters are two things unknown, not one moment observed twice.
        self.count.is_some() && self == later && !self.writer_active
    }
}

/// Held for the mutating part of a command. Dropping it releases the lock.
pub struct DataLock {
    file: File,
    owner_path: PathBuf,
    data_dir: PathBuf,
}

impl DataLock {
    /// Take the lock from an `async` command, without parking a runtime worker.
    ///
    /// The wait below is `thread::sleep` in a poll loop for up to two minutes, and every caller
    /// is inside `#[tokio::main]`. `run_exclusive` already moved its `flock` wait to the blocking
    /// pool for exactly this reason and wrote down why; this is the same wait, one layer up,
    /// which nobody had noticed was the same.
    pub async fn acquire_async(data_dir: &Path, command: &str, timeout: Duration) -> Result<Self> {
        let dir = data_dir.to_path_buf();
        let command = command.to_string();
        crate::core::off_the_runtime(move || Self::acquire(&dir, &command, timeout)).await?
    }

    /// Take the lock if it is free at this instant, or report that somebody else holds it.
    ///
    /// **For a caller with nothing to do with the wait.** A `hook-*` subcommand is fired by a
    /// manager mid-transaction; if the directory is locked, the run holding it is the run that
    /// is going to record what the manager just did, so waiting two minutes to be told so
    /// costs the transaction two minutes and changes nothing. This returns `None` instead,
    /// and says nothing â€” contention is the ordinary case here, not a fault.
    pub fn try_acquire(data_dir: &Path, command: &str) -> Result<Option<Self>> {
        let (file, owner_path) = Self::open_lock_file(data_dir)?;
        match file.try_lock_exclusive() {
            Ok(()) => Ok(Some(Self::stamped(file, owner_path, data_dir, command))),
            Err(e) if is_lock_contended(&e) => Ok(None),
            Err(e) => Err(Error::Io(format!(
                "could not lock {}: {}",
                data_dir.join("shall.lock").display(),
                e
            ))),
        }
    }

    /// Open the lock file and name its owner stamp. Shared so the waiting and non-waiting
    /// doors cannot disagree about which file in which directory is the lock.
    fn open_lock_file(data_dir: &Path) -> Result<(File, PathBuf)> {
        crate::utils::file::ensure_dir(data_dir)?;
        let path = data_dir.join("shall.lock");
        let owner_path = data_dir.join("shall.lock.owner");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(Error::from)?;
        Ok((file, owner_path))
    }

    /// Record who holds it. Written after the lock is taken, so the stamp cannot name a
    /// process that failed to get it.
    fn stamped(file: File, owner_path: PathBuf, data_dir: &Path, command: &str) -> Self {
        let stamp = format!("shall {} (pid {})", command, std::process::id());
        let _ = std::fs::write(&owner_path, stamp);
        HELD.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        Self {
            file,
            owner_path,
            data_dir: data_dir.to_path_buf(),
        }
    }

    /// Take the lock for one write, unless this process is already inside it.
    ///
    /// **The door for code that cannot know whether its caller holds the lock** â€” a ledger
    /// save reached from `sync` is covered by the run's own lock, and the same save reached
    /// from `check` is covered by nothing. Asking the caller to pass a token down twenty call
    /// sites answers this at compile time, which is the wrong time: `Deferred` releases the
    /// lock between actions, so the answer changes during a run.
    ///
    /// `Ok(None)` means the lock is already this process's and the caller writes as it is;
    /// re-taking it would be `flock` waiting for the same process's other handle, for ever.
    pub fn for_this_write(what: &str) -> Result<Option<Self>> {
        if held() {
            return Ok(None);
        }
        Self::acquire(
            &crate::utils::safe_data_dir(),
            what,
            Duration::from_secs(WAIT_SECS),
        )
        .map(Some)
    }

    /// Take the lock, waiting up to `timeout` for whoever holds it.
    ///
    /// Waiting with no reason given is indistinguishable from hanging, so the wait announces
    /// the holder â€” the lock file carries the pid and the command that took it.
    pub fn acquire(data_dir: &Path, command: &str, timeout: Duration) -> Result<Self> {
        let (file, owner_path) = Self::open_lock_file(data_dir)?;
        let path = data_dir.join("shall.lock");

        match file.try_lock_exclusive() {
            Ok(()) => {}
            Err(e) if is_lock_contended(&e) => {
                eprintln!(
                    "shall: waiting for the data directory â€” held by {}",
                    Self::holder(&owner_path)
                );
                let deadline = Instant::now() + timeout;
                loop {
                    match file.try_lock_exclusive() {
                        Ok(()) => break,
                        Err(e) if !is_lock_contended(&e) => {
                            return Err(Error::Io(format!(
                                "could not lock {}: {}",
                                path.display(),
                                e
                            )));
                        }
                        Err(_) => {}
                    }
                    if Instant::now() >= deadline {
                        // S27: the old text ended "remove shall.lock if nothing is running", and
                        // that advice is never right. The lock is an OS lock on an open handle,
                        // released when the holding process exits â€” so a lock that is still
                        // contended after the wait proves a live holder, and deleting the file
                        // takes the lock away from it rather than from a corpse.
                        return Err(Error::Other(format!(
                            "the Shall data directory is locked by {}, and still was after {}s.\n  \
                             {} is where state lives, and two writers make a removal out of a race.\n  \
                             The lock is held by a running process, not by the file: {} exists\n  \
                             between runs and deleting it would take the lock from a live writer.\n  \
                             Wait for that run to finish, or stop it.",
                            Self::holder(&owner_path),
                            timeout.as_secs(),
                            data_dir.display(),
                            path.display()
                        )));
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
            Err(e) => {
                return Err(Error::Io(format!(
                    "could not lock {}: {}",
                    path.display(),
                    e
                )));
            }
        }

        Ok(Self::stamped(file, owner_path, data_dir, command))
    }

    /// Take the lock for one mutating step of a command that does not hold it for its run.
    ///
    /// **The one place the wait and the directory are written down.** Three call sites had
    /// copied `safe_data_dir()`, `Duration::from_secs(120)` and the name-it-yourself argument,
    /// which is three chances for the wait to disagree with `main`'s and a fourth caller to
    /// invent a fifth number. `LockScope::Deferred` is what says a command belongs here.
    pub async fn for_one_step(what: &str) -> Result<Self> {
        Self::acquire_async(
            &crate::utils::safe_data_dir(),
            what,
            Duration::from_secs(WAIT_SECS),
        )
        .await
    }

    /// Take the lock for one step if it is free, standing down rather than waiting.
    ///
    /// Beside [`for_one_step`](Self::for_one_step) so the directory stays written down once:
    /// a caller that spelled `safe_data_dir()` itself would be the fourth copy the doc on
    /// that function is about.
    pub fn try_for_one_step(what: &str) -> Result<Option<Self>> {
        Self::try_acquire(&crate::utils::safe_data_dir(), what)
    }

    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    /// Who is holding the lock, for the message. The stamp lives beside the lock file rather
    /// than inside it: Windows refuses to read a file another process holds an exclusive lock
    /// on, which would leave the one message this exists to print saying nothing.
    pub fn holder(owner_path: &Path) -> String {
        match std::fs::read_to_string(owner_path) {
            Ok(s) if !s.trim().is_empty() => s.trim().to_string(),
            _ => "another shall".to_string(),
        }
    }
}

impl Drop for DataLock {
    fn drop(&mut self) {
        // Bumped before the stamp goes and before the lock is released, so a reader that sees
        // no writer and an unchanged count is reading after this one's writes, never during.
        //
        // **A preview does not bump it**, and that is not an exemption from S25 but the rule
        // itself: this counter says "a writer committed something", and a run that wrote
        // nothing has nothing for a reader to detect. Writing it anyway would also be a
        // preview leaving a file behind, which is the defect the whole dry-run rule exists to
        // prevent â€” `a_preview_leaves_the_config_byte_identical` caught exactly that here.
        if !crate::core::dry_run::active() {
            // Atomically, through the writer this repo requires everywhere else. The one raw
            // `fs::write` in the tree was this one, and a crash inside it left a torn file â€”
            // which is the whole of why the read above has to distinguish torn from absent.
            //
            // From an unreadable counter there is no right number, only a number unlikely to
            // equal one a reader has already seen â€” and the whole point of the bump is that the
            // value *moves*. `u64::MAX` is that; guessing `1` is a value a young data directory
            // really has.
            let next = observe(&self.data_dir)
                .count
                .map_or(u64::MAX, |c| c.wrapping_add(1));
            let _ = crate::utils::file::persist(
                &self.data_dir.join(GENERATION_FILE),
                &next.to_string(),
            );
        }
        HELD.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
        let _ = std::fs::remove_file(&self.owner_path);
        // The lock file itself stays. Deleting it races the next process, which may already
        // have opened this inode and be about to lock a file no longer at that name.
        let _ = FileExt::unlock(&self.file);
    }
}
