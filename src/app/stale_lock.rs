//! The package manager's own lock, left behind by a run that was killed (`Q50`).
//!
//! **The failure.** A sync is interrupted — Ctrl-C, a dead battery, a container torn down mid
//! transaction — and the manager underneath it dies with its lock file still on disk. From then
//! on *every* Shall run on that machine fails, with the manager's own words:
//!
//! ```text
//! error: failed to init transaction (unable to lock database)
//! error: could not lock database: File exists
//!   if you're sure a package manager is not already running, you can remove
//!   /var/lib/pacman/db.lck
//! ```
//!
//! Shall already says this well — it relays the advice and adds *"tried 4 times; the failure did
//! not change, so this is not the transient failure its output looks like"*. What it could not do
//! is act, and `heal` is the command whose entire job is *a run was interrupted, make this
//! machine workable again*.
//!
//! **Only locks whose existence IS the lock.** This is the distinction the whole module turns on,
//! and getting it wrong corrupts a package database:
//!
//! | manager | lock | may it be removed? |
//! |---|---|---|
//! | pacman | `/var/lib/pacman/db.lck` | **yes** — created on start, removed on exit |
//! | dnf | `/var/cache/dnf/metadata_lock.pid` | **yes** — a pid file, removed on exit |
//! | zypper | `/run/zypp.pid` | **yes** — a pid file, removed on exit |
//! | apt / dpkg | `/var/lib/dpkg/lock-frontend`, `/var/lib/apt/lists/lock` | **NO** |
//!
//! apt and dpkg lock with `flock(2)` on files that exist permanently, whether or not anything
//! holds them. Their presence says nothing at all, the kernel drops the lock when the holder
//! dies, and deleting them removes the very files the next `apt` expects to lock. They are rows
//! in the same table rather than a second list beside it: a row that says *why it may never be
//! removed* travels with the paths it is about, and cannot be re-admitted by someone extending
//! the table who never read the other one.
//!
//! **Staleness is proved, never assumed.** A lock carrying a pid is stale when that pid is not
//! running. A lock carrying nothing is stale when no process of the owning manager is running at
//! all. Neither question is asked on a platform that has no `/proc`, and every one of these
//! managers is Linux-only, so elsewhere this module finds nothing and says nothing.
//!
//! **And the other half of the same knowledge: waiting.** A lock that a *live* manager holds is
//! not a defect at all — it is the ordinary case of two package managers on one machine, and the
//! only thing that helps is to wait for the one that got there first. Shall used to retry four
//! times over three and a half seconds and then tell the user *"this is not the transient failure
//! its output looks like"*, which was false in exactly the case it was printed. [`held_for`]
//! answers which of the three states the machine is in, so the retry loop can wait, refuse, or
//! send the user to `heal` — and say something true in each case.

use std::path::{Path, PathBuf};

/// A manager lock, what proves it is not held, and whether it may ever be cleared.
pub struct ManagerLock {
    /// Every binary that could be holding it, display name first. More than one because a
    /// manager is rarely one program: `apt`, `apt-get` and `dpkg` share dpkg's lock, and an
    /// `unattended-upgrade` holds it without either name on the command line.
    pub holders: &'static [&'static str],
    /// The backends whose failures this lock explains. `pacman`'s lock is what stops `yay`.
    pub backends: &'static [&'static str],
    /// Where the lock lives. Several, because a manager may leave more than one.
    pub paths: &'static [&'static str],
    /// Whether the file's contents are the pid of the holder.
    pub carries_pid: bool,
    /// Why this lock may never be removed — `None` when its existence *is* the lock and
    /// clearing a stale one is the repair.
    ///
    /// A row carrying a reason is still a full row: it is matched, waited on, and reported. The
    /// only thing it never is, is deleted.
    pub never_remove_because: Option<&'static str>,
    /// What the manager prints when someone else holds this lock. Lowercase — matched against
    /// the lowercased failure text.
    pub taken_markers: &'static [&'static str],
}

impl ManagerLock {
    /// The name to put in a sentence.
    pub fn holder(&self) -> &'static str {
        self.holders[0]
    }

    /// Whether a stale one of these may be cleared.
    pub fn removable(&self) -> bool {
        self.never_remove_because.is_none()
    }
}

/// Every lock whose *presence* means a run is in progress.
///
/// A manager joins this list when someone has checked that its lock is created and deleted
/// around the transaction rather than existing permanently — the same standard
/// `core::argv`'s terminator table holds, and for the same reason: the cost of being wrong is
/// paid on somebody's machine, not here.
pub const MANAGER_LOCKS: &[ManagerLock] = &[
    ManagerLock {
        holders: &["pacman"],
        // The AUR helpers drive pacman for the write, so pacman's lock is what stops them, and
        // the process holding it is called `pacman` whichever of them started it. **This list is
        // what decides a lock is stale**, so a helper that held it under its own name would be a
        // live lock deleted — asked rather than reasoned about, in the arch image: 435 samples
        // taken while `yay -Sy` and `paru -Sy` held the lock, a live `pacman` in every one, none
        // with the helper running and no pacman anywhere.
        backends: &["pacman", "yay", "paru"],
        paths: &["/var/lib/pacman/db.lck"],
        // pacman's db.lck is empty. Its own message says so: *"if you're sure a package manager
        // is not already running"* — it cannot tell you itself, because it wrote no pid.
        carries_pid: false,
        never_remove_because: None,
        taken_markers: &["unable to lock database", "could not lock database"],
    },
    ManagerLock {
        holders: &["dnf", "dnf5", "yum", "microdnf"],
        backends: &["dnf", "yum", "microdnf"],
        paths: &["/var/cache/dnf/metadata_lock.pid"],
        carries_pid: true,
        never_remove_because: None,
        taken_markers: &[
            "another app is currently holding the yum lock",
            "waiting for process with pid",
        ],
    },
    ManagerLock {
        holders: &["zypper"],
        backends: &["zypper"],
        paths: &["/run/zypp.pid"],
        carries_pid: true,
        never_remove_because: None,
        taken_markers: &[
            "system management is locked by the application",
            "zypp-refresh",
        ],
    },
    // The `flock(2)` family. Present always, held only sometimes, and never removable — but
    // still a row, because "who holds it and should we wait" is a question worth asking about
    // apt more than about any other manager on this list.
    ManagerLock {
        holders: &["apt", "apt-get", "dpkg", "unattended-upgr", "aptd"],
        backends: &["apt", "apt-get", "dpkg"],
        paths: &[
            "/var/lib/dpkg/lock-frontend",
            "/var/lib/dpkg/lock",
            "/var/lib/apt/lists/lock",
            "/var/cache/apt/archives/lock",
        ],
        carries_pid: false,
        never_remove_because: Some(
            "dpkg and apt lock these with flock(2). The files are permanent, their presence \
             means nothing, the kernel released the lock the moment the holder died, and \
             deleting one deletes what the next `apt` expects to lock",
        ),
        taken_markers: &[
            "could not get lock",
            "unable to acquire the dpkg frontend lock",
            "is another process using it",
        ],
    },
];

/// What a stale lock is, once found: enough to report it and to remove it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stale {
    pub path: PathBuf,
    pub holder: &'static str,
    /// The sentence `heal` prints. Carried rather than rebuilt so the report and the decision
    /// cannot describe different things.
    pub because: String,
}

/// A lock that is there and is being left alone, with the reason.
///
/// **`heal` is never silent about a lock it examined.** The first version reported only what it
/// removed, so the run where it decided *not* to — the one that then failed on that very lock —
/// printed nothing at all, and the reason had to be inferred from a machine where the decision
/// went the other way. Twice, wrongly. A decision this consequential says itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeftAlone {
    pub path: PathBuf,
    pub because: String,
}

/// Everything the scan concluded: what may go, and what may not and why.
#[derive(Debug, Default)]
pub struct Survey {
    pub stale: Vec<Stale>,
    pub left: Vec<LeftAlone>,
}

/// Ask the running system whether a process is there.
///
/// A trait rather than a direct `/proc` read so the decision can be tested against a machine
/// that is not this one — the whole risk here is answering "not running" about a manager that
/// is, and a rule that can only be exercised by killing a real pacman is a rule nobody exercises.
pub trait Processes {
    /// Is a process with this pid alive?
    fn pid_alive(&self, pid: u32) -> bool;
    /// Is any process running under this program name?
    fn any_named(&self, name: &str) -> bool;
}

/// **A zombie holds nothing.** It is a process that has already exited and whose parent has not
/// yet collected its status; the kernel released every file it had open at the moment it died.
/// It nonetheless keeps its `/proc/<pid>` entry, and `comm` still reads `pacman`.
///
/// This cost a green CI run. A killed sync leaves its `pacman` reaped a moment later, and `heal`
/// runs inside that moment — asking "is a pacman running", getting yes from a corpse, and
/// leaving the lock. The scan that said no pacman was there looked nine seconds afterwards,
/// which is how it read as a contradiction rather than as a race.
///
/// Field 3 of `/proc/<pid>/stat` is the state letter. It is read after the last `)` because a
/// process name can contain spaces and parentheses — `(pac man)` would shift every field if the
/// line were split from the left.
/// **`pub` for the integration tests' sake.** The stale-lock mirror links the library without
/// `cfg(test)` and must exercise the real `/proc` parsing seam.
pub fn is_zombie(proc_dir: &Path) -> bool {
    std::fs::read_to_string(proc_dir.join("stat"))
        .ok()
        .and_then(|stat| {
            let after_name = stat.rsplit_once(')')?.1;
            after_name.split_whitespace().next().map(str::to_string)
        })
        .is_some_and(|state| state == "Z")
}

/// The real answer, from `/proc`. Nothing else has to be installed — `pgrep` is absent from
/// several of the images these managers run on.
pub struct ProcFs;

impl Processes for ProcFs {
    fn pid_alive(&self, pid: u32) -> bool {
        Path::new(&format!("/proc/{pid}")).is_dir()
    }

    fn any_named(&self, name: &str) -> bool {
        let Ok(entries) = std::fs::read_dir("/proc") else {
            // No `/proc`: this is not Linux, and none of these managers runs here. Answering
            // "yes, something is running" is the safe direction — it clears nothing.
            return true;
        };
        entries.flatten().any(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.chars().all(|c| c.is_ascii_digit()))
                && std::fs::read_to_string(e.path().join("comm"))
                    .is_ok_and(|comm| comm.trim() == name)
                && !is_zombie(&e.path())
        })
    }
}

/// What the machine says about a manager's lock right now.
///
/// The three answers are three different actions, and collapsing any two of them is how the
/// wrong sentence got printed: Shall treated *held by a running manager* and *left behind by a
/// dead one* as one failure, retried both four times in three and a half seconds, and told the
/// user neither.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Held {
    /// A live process holds it. Nothing is wrong; the work is not lost; waiting is the fix.
    Live(String),
    /// The lock is on disk and nothing holds it. Waiting is forever; `heal` is the fix.
    Stale(PathBuf),
    /// Nothing holds it. If a command just failed on it, the holder let go in between.
    Free,
}

/// Which lock, if any, explains a failure from this backend.
pub fn lock_of(backend: &str) -> Option<&'static ManagerLock> {
    MANAGER_LOCKS.iter().find(|l| l.backends.contains(&backend))
}

/// The name a manager's verbs take the exclusive lock under.
///
/// **One key per package database, not per program.** OpenBSD installs with `pkg_add` and
/// removes with `pkg_delete`, and keying on the program gave those two verbs two different
/// locks over one database; `pacman` and `yay` in one config were two locks over one database,
/// so a sync touching both ran them concurrently and let pacman's own `db.lck` decide, which it
/// does by failing the loser.
///
/// **The one place that answers this.** `GenericBackendCore::lock_key` used to be it, and the
/// hand-written backends each spelled their key as a literal instead — `run_exclusive("brew",
/// "brew", …)`. Every literal happened to equal what this returns, because none of brew,
/// flatpak, go, mise, nix or emacs is in a lock family; that is the definition of a second copy
/// of a table waiting to go stale, which is what the table's own doc says about second copies.
pub fn lock_key(backend: &str) -> &str {
    lock_of(backend).map_or(backend, |l| l.holder())
}

/// Whether this failure text is the manager saying someone else holds its lock.
///
/// Asked of the failure Shall already has rather than of the filesystem, because the manager is
/// the only one that knows which of its locks it wanted. A backend with no lock in the table
/// answers `false` and pays one list scan for it.
pub fn says_the_lock_is_taken(backend: &str, message: &str) -> bool {
    let Some(lock) = lock_of(backend) else {
        return false;
    };
    let hay = message.to_ascii_lowercase();
    lock.taken_markers.iter().any(|m| hay.contains(m))
}

/// Who holds `backend`'s manager lock, as the machine can prove it.
///
/// The `flock(2)` rows cannot be answered from the filesystem at all — the file is there whether
/// or not it is held — so for those the running-process question *is* the whole answer, and it is
/// a sound one: the kernel drops an `flock` when its holder dies, so a lock that is genuinely
/// taken has a live taker by definition. Those rows can therefore never report [`Held::Stale`],
/// which is the same fact as their never being removable, arrived at from the other side.
///
/// **No lock file on disk, no lock.** A running `pacman` with no `db.lck` is a `pacman` doing
/// something that is not a transaction, and there is nothing to wait for. Asking the process list
/// first got this backwards, and the cost was not theoretical: `ProcFs::any_named` answers *yes*
/// on a machine with no `/proc` — deliberately, because for *clearing* a lock that is the safe
/// direction — so on Windows every row read as held, and the settle step `heal` runs would have
/// waited the full budget on each of four locks that do not exist. Twenty minutes of nothing.
/// What looking at a lock file found.
///
/// **Three states, because two of them were sharing a spelling and it cost a machine its
/// recovery.** Both readers here took `Option<String>` and treated `None` as *absent*, and
/// `None` is also what an existing file returns when the caller may not read it. pacman's
/// `db.lck` is created mode `0000` and owned by root; every command Shall sends pacman is
/// elevated, but the *look* at the lock is not — so as an unprivileged user a stale lock read
/// as no lock at all.
///
/// Measured in the arch image, which is the one harness that runs as a normal user: a SIGKILL
/// mid-transaction left `db.lck` behind, `heal` said *"left it alone — it is there and could
/// not be read, so nothing can be proved about it"*, `held_for` reported `Free`, and every sync
/// afterwards exhausted its retries on `could not lock database: File exists`. The machine
/// could not be recovered by any Shall command, on a lock Shall already knew how to remove —
/// `clear_stale_manager_locks` runs `rm -f` through the elevating executor.
///
/// **Unreadable is only evidence for a lock that carries a pid.** pacman's is empty by design;
/// its own error message says so, because it wrote no pid to read. Demanding readable contents
/// that are then never used is asking for a permission in order to ignore what it grants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockFile {
    /// Not on disk. No lock.
    Absent,
    /// On disk, and its contents could not be read.
    Unreadable,
    /// On disk and read; the body may be empty.
    Body(String),
}

impl LockFile {
    /// Whether a file is there at all, whatever could be read of it.
    pub fn present(&self) -> bool {
        !matches!(self, LockFile::Absent)
    }
}

/// Look at a path on the real filesystem. `exists()` is a `stat`, which needs a traversable
/// parent and not a readable file — which is the whole distinction being drawn.
fn look(p: &Path) -> LockFile {
    match std::fs::read_to_string(p) {
        Ok(body) => LockFile::Body(body),
        Err(_) if p.exists() => LockFile::Unreadable,
        Err(_) => LockFile::Absent,
    }
}

pub fn held_for(backend: &str, procs: &dyn Processes, read: &dyn Fn(&Path) -> LockFile) -> Held {
    let Some(lock) = lock_of(backend) else {
        return Held::Free;
    };
    let present: Vec<&Path> = lock
        .paths
        .iter()
        .map(Path::new)
        .filter(|p| read(p).present())
        .collect();
    if present.is_empty() {
        return Held::Free;
    }
    if let Some(who) = running_holder(lock, procs) {
        return Held::Live(who);
    }
    if !lock.removable() {
        // Nothing is running, and the file's presence proves nothing. There is no stale
        // `flock` to report and nothing to wait for.
        return Held::Free;
    }
    for path in present {
        // The existence check is the answer for a pid-less lock, so it is made before the read
        // rather than behind it: nothing below reads `body` on that branch.
        if !lock.carries_pid {
            // Its existence is the lock, and nothing of the manager's is running.
            return Held::Stale(path.to_path_buf());
        }
        let LockFile::Body(body) = read(path) else {
            continue;
        };
        // A pid file still naming a live process is held even when no process carries the
        // manager's name — `dnf` behind PackageKit is the case, and it is why the pid is read
        // rather than assumed to agree with the process list.
        match body.trim().parse::<u32>() {
            Ok(pid) if procs.pid_alive(pid) && !pid_recycled_since(pid, file_mtime(path)) => {
                return Held::Live(format!("pid {pid}"))
            }
            // Alive but born after the lock was written: the number was reused, and the
            // manager that took this lock is gone.
            Ok(_) => return Held::Stale(path.to_path_buf()),
            // Half-written by a process that died between `create` and `write`. That proves
            // nothing, and "proves nothing" must not become "go and delete it" — `find` leaves
            // exactly this file alone, and an answer here that sent the user to `heal` would
            // send them to a command that then declines.
            Err(_) => return Held::Free,
        }
    }
    Held::Free
}

/// The same question against the real machine.
pub fn held_for_on_this_machine(backend: &str) -> Held {
    held_for(backend, &ProcFs, &look)
}

/// How a wait for someone else's manager ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Waited {
    /// It let go. Whether it finished or died is the next question's business, not this one's.
    Freed(std::time::Duration),
    /// Still holding when the budget ran out. Nothing is wrong; it is simply still working.
    StillHeld,
    /// The run was cancelled underneath the wait.
    Cancelled,
}

/// Wait until nothing live holds `backend`'s manager lock.
///
/// One polling loop, because there are two callers and they must agree: the retry loop waits
/// here before trying a command again, and `heal` waits here before deciding whether a lock is
/// stale. `heal` learned that the hard way — it surveyed once at the top, correctly left a lock
/// alone because a `pacman` was alive, and then watched that `pacman` exit during the recovery
/// it went on to run. By the time the lock was stale, the only step that could clear it had
/// already happened.
pub async fn wait_until_not_held(
    backend: &str,
    budget: std::time::Duration,
    cancelled: &(dyn Fn() -> bool + Sync),
) -> Waited {
    let started = std::time::Instant::now();
    while started.elapsed() < budget {
        if cancelled() {
            return Waited::Cancelled;
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if !matches!(held_for_on_this_machine(backend), Held::Live(_)) {
            return Waited::Freed(started.elapsed());
        }
    }
    Waited::StillHeld
}

/// The first of a lock's holder programs that is running, phrased for a sentence.
fn running_holder(lock: &ManagerLock, procs: &dyn Processes) -> Option<String> {
    lock.holders
        .iter()
        .find(|h| procs.any_named(h))
        .map(|h| format!("a `{h}`"))
}

fn file_mtime(p: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

/// Whether `pid` came into existence AFTER the lock file was written.
///
/// **The wraparound guard on bare pid-liveness.** Pids are reused; a lock naming 4132 whose
/// dnf died yesterday reads as held by whatever unrelated process is 4132 today, for as long
/// as that process happens to run — which can be indefinitely. The process's own start time
/// (from `/proc/{pid}/stat`) settles it: a holder predates the lock it took. Best-effort in
/// the SAFE direction — anything unparseable answers "not recycled", which is the old
/// behaviour.
#[cfg(target_os = "linux")]
fn pid_recycled_since(pid: u32, since: Option<std::time::SystemTime>) -> bool {
    recycled_since_opt(pid, since).unwrap_or(false)
}

/// `None` is every unanswerable case — unreadable `/proc`, a comm field with no closing
/// paren, a tick count that does not parse — and the caller reads it as "not recycled".
#[cfg(target_os = "linux")]
fn recycled_since_opt(pid: u32, since: Option<std::time::SystemTime>) -> Option<bool> {
    let since = since?;
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // Everything after the comm field — comm may contain spaces and parens of its own.
    let after_comm = stat.rfind(')').map(|i| &stat[i + 1..])?;
    let fields: Vec<&str> = after_comm.split_whitespace().collect();
    // Field 22 overall; two (state, ppid) precede this slice, so starttime is index 19 here.
    let ticks: u64 = fields.get(19)?.parse().ok()?;
    const CLK_TCK: f64 = 100.0; // userspace Hz on every Linux this ships to
    let started_after_boot = std::time::Duration::from_secs_f64(ticks as f64 / CLK_TCK);
    let uptime: f64 = std::fs::read_to_string("/proc/uptime")
        .ok()?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    let boot =
        std::time::SystemTime::now().checked_sub(std::time::Duration::from_secs_f64(uptime))?;
    let started_at = boot.checked_add(started_after_boot)?;
    Some(started_at > since)
}

#[cfg(not(target_os = "linux"))]
fn pid_recycled_since(_pid: u32, _since: Option<std::time::SystemTime>) -> bool {
    false
}

/// Every manager lock on this machine that exists and is provably not held.
///
/// `read` is how the file's contents are obtained, so the decision can be tested without a
/// filesystem. Nothing here removes anything — finding and acting are separate so `heal
/// --dry-run` can report exactly what the real run would do.
pub fn find(procs: &dyn Processes, read: &dyn Fn(&Path) -> LockFile) -> Survey {
    let mut out = Survey::default();
    for lock in MANAGER_LOCKS {
        // The `flock(2)` rows are skipped whole, and silently. They are present on every Debian
        // machine that ever ran `apt`, so reporting them as "left alone" would print four lines
        // of noise on every `heal` — and the one thing that must never happen to them, being
        // removed, cannot happen if they never reach the loop below.
        if !lock.removable() {
            continue;
        }
        for path in lock.paths {
            let path = Path::new(path);
            let body = match read(path) {
                LockFile::Absent => continue,
                // A lock that is there and unreadable is not a lock to be silent about — it is
                // the strongest hint a reader could get about why a manager is refusing. But
                // it is only *undecidable* where the contents were going to decide it: a
                // pid-less lock is answered by its existence and the process list, neither of
                // which needs the file opened. Treating those as unprovable left pacman's
                // `db.lck` in place after every killed run on a machine where Shall was not
                // root, and the elevated `rm` that would have cleared it was never reached.
                LockFile::Unreadable if lock.carries_pid => {
                    out.left.push(LeftAlone {
                        path: path.to_path_buf(),
                        because: "it should name a pid and could not be read, so nothing can be \
                                  proved about it"
                            .into(),
                    });
                    continue;
                }
                LockFile::Unreadable => String::new(),
                LockFile::Body(body) => body,
            };
            let because = match (lock.carries_pid, body.trim().parse::<u32>()) {
                // A pid that is still running AND predates the lock: held, and this is the
                // case the whole module exists to not get wrong. The start-time check is the
                // wraparound guard — a reused pid answers alive, but it was not born when
                // this lock was taken.
                (true, Ok(pid))
                    if procs.pid_alive(pid)
                        && !pid_recycled_since(pid, file_mtime(Path::new(path))) =>
                {
                    out.left.push(LeftAlone {
                        path: path.to_path_buf(),
                        because: format!("pid {pid} is running and holds it"),
                    });
                    continue;
                }
                (true, Ok(pid)) => format!(
                    "it names pid {pid}, and no such process is running — the run that took it \
                     was killed"
                ),
                // A pid file with no readable pid is not evidence of anything; leave it.
                (true, Err(_)) => {
                    out.left.push(LeftAlone {
                        path: path.to_path_buf(),
                        because: "it should name a pid and does not, which proves nothing either \
                                  way"
                        .into(),
                    });
                    continue;
                }
                (false, _) => match running_holder(lock, procs) {
                    Some(who) => {
                        out.left.push(LeftAlone {
                            path: path.to_path_buf(),
                            because: format!("{who} is running, so it is held"),
                        });
                        continue;
                    }
                    None => format!(
                        "it carries no pid and no `{}` is running, so nothing holds it",
                        lock.holder()
                    ),
                },
            };
            out.stale.push(Stale {
                path: path.to_path_buf(),
                holder: lock.holder(),
                because,
            });
        }
    }
    out
}

/// The same question against the real machine.
pub fn find_on_this_machine() -> Survey {
    find(&ProcFs, &look)
}
