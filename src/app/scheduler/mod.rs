use crate::config::config::ScheduleConfig;
use crate::core::{CommandExecutor, Error, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Delete a unit file Shall generated. A file that is already gone is the wanted end state;
/// any other failure leaves a schedule armed that Shall is about to report as removed.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn remove_generated(path: &Path) -> Result<()> {
    crate::utils::file::force_remove(path).map_err(|e| {
        Error::Io(format!(
            "{e}. The schedule may still be armed; remove the file by hand and re-run \
             `shall sync`."
        ))
    })
}

/// What a scheduler holds for one task — and, from [`TaskProvisioner::rendered`], what Shall
/// would put there for a declaration.
///
/// `spec` is only ever compared against another `spec` from the **same** provisioner. A systemd
/// unit file and a Task Scheduler trigger have nothing to say to each other, and the value of
/// the comparison is that each provisioner gets to make it in its own terms: systemd and launchd
/// compare the whole generated file, so every option they can express is covered without anyone
/// remembering to add it here, and Task Scheduler — which keeps no file — compares a canonical
/// form built from the trigger XML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provisioned {
    /// The scheduler's whole record of the task, in that scheduler's own spelling.
    pub spec: String,
    /// Whether it fires. A task that exists and is disabled is not the schedule that was
    /// declared, and nothing that only asks "does it exist" can tell the difference.
    pub armed: bool,
}

/// The answer to *what does this machine hold for this schedule*.
///
/// **`Unreadable` is not `Absent`.** A read that failed reported as "not there" would re-write
/// the schedule on every sync for ever and keep `check` permanently red on something nobody can
/// see — the rule V.188 states for `setting:`, which applies here for the same reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reading {
    /// The scheduler answered, and this is what it holds.
    Holds(Provisioned),
    /// The scheduler answered, and it holds nothing under that name.
    Absent,
    /// The scheduler could not be asked.
    Unreadable,
}

/// How the machine stands against one `schedule:` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    /// The scheduler holds exactly what the declaration says.
    Matches,
    /// Nothing is provisioned under that name.
    Missing,
    /// Something is provisioned and it is not this. The string says what changed, for the line
    /// `sync` prints before it re-provisions.
    Differs(&'static str),
    /// It could not be asked, so nothing is claimed either way.
    Unknown,
}

impl Standing {
    /// The three-valued answer `apply::extras::in_effect` speaks: `Some(true)` in effect,
    /// `Some(false)` work to do, `None` unverifiable.
    pub fn in_effect(&self) -> Option<bool> {
        match self {
            Standing::Matches => Some(true),
            Standing::Missing | Standing::Differs(_) => Some(false),
            Standing::Unknown => None,
        }
    }
}

/// The separator between the two systemd units in one `spec`. A `#` line is a comment in unit
/// syntax, so a spec that ever did reach a file would still parse.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub const UNIT_SEPARATOR: &str = "\n#--- shall: timer ---\n";

/// Cap on the scheduled-run log, and one generation of history.
///
/// `schedule.log` is Shall-generated — systemd's `append:` and launchd's `StandardOutPath`
/// both point here — and nothing rotated it: a nightly sync on a chatty mirror wrote until
/// the disk noticed. Called from `main` before anything else runs, because every writer of
/// this file is a shall process. One `.1` generation keeps yesterday's failure readable
/// after a rotation; deeper history is what the journal is for.
///
/// **This is the one file every Shall process touches, and it is touched from `main` before
/// the data lock**, so rotation takes a lock of its own and gives up when somebody else holds
/// it. Two runs started together — a timer and a person typing `shall plan` — both saw a log
/// over the cap; the old code's `remove_file(schedule.log.1)` then deleted the generation the
/// other process had just set aside, and the loser's `rename` failed and printed
/// `could not rotate schedule.log` on a command that had nothing to do with the log.
///
/// **`rename` alone replaces the destination** on POSIX and on Windows alike (Rust's
/// `fs::rename` is `MoveFileEx` with `MOVEFILE_REPLACE_EXISTING` there), so the `remove_file`
/// was never needed — it was the whole of the loss.
///
/// The notice is `-v` only: a rotation is bookkeeping, and it used to arrive on the stderr of
/// whichever command happened to run first, most often an interactive `shall search`.
pub fn rotate_log_if_large(verbose: bool) {
    const MAX_BYTES: u64 = 10 * 1024 * 1024;
    let data_dir = crate::utils::safe_data_dir();
    let log = data_dir.join("schedule.log");
    let Ok(meta) = std::fs::metadata(&log) else {
        return;
    };
    if meta.len() <= MAX_BYTES {
        return;
    }
    // A separate file, not the log: the lock has to outlive the rename that moves the log's
    // inode, and on Windows an open handle on the file being renamed is a sharing violation.
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(data_dir.join("schedule.log.lock"))
        .ok();
    let Some(lock) = lock else {
        return;
    };
    // Non-blocking: a process already rotating this log is doing the work this call exists
    // for, and a `shall plan` that waits on it is a reader waiting on bookkeeping.
    if fs2::FileExt::try_lock_exclusive(&lock).is_err() {
        return;
    }
    let rotated = log.with_extension("log.1");
    match std::fs::rename(&log, &rotated) {
        Ok(()) => {
            if verbose {
                eprintln!("schedule.log passed 10 MiB and was set aside as schedule.log.1");
            }
        }
        Err(e) => {
            if verbose {
                eprintln!("could not rotate schedule.log: {e}");
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskTarget {
    pub account: Option<crate::core::account::Account>,
    /// The config and data roots the **declaring** run used, so a firing run resolves the same
    /// machine description. Default (empty) means the defaults then, so a legacy task written
    /// before this field existed is spelled exactly as it was.
    pub roots: Roots,
}

/// The roots a run was declared from — the two flags a firing task needs to fire against the
/// same store it was declared from (#192).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Roots {
    pub config: PathBuf,
    pub data: PathBuf,
}

impl Roots {
    /// The roots of the run that is declaring: what it resolved, not what the OS default is.
    pub fn of(config: &crate::config::Config) -> Self {
        Self {
            config: config.config_root(),
            data: config.data_root(),
        }
    }

    /// `--config-dir <root>` when the run did not use the OS default, nothing otherwise. Same
    /// rule and same absences for data — a common-case task is spelled exactly as it was, and a
    /// non-default task cannot forget itself.
    fn flag(&self, which: &'static str, root: &Path, default: &Path) -> Option<String> {
        if root == default {
            return None;
        }
        let value = if root.as_os_str().is_empty() {
            return None;
        } else if root.to_string_lossy().contains(char::is_whitespace) {
            // Double quotes, the quoting both systemd's ExecStart and schtasks' /TR parse in
            // their own (non-shell) way. Bare is fine for launchd because it is an argv array,
            // not a command line.
            format!("{which} \"{}\"", root.display())
        } else {
            format!("{which} {}", root.display())
        };
        Some(value)
    }

    fn config_flag(&self) -> Option<String> {
        self.flag(
            "--config-dir",
            &self.config,
            &crate::utils::safe_config_dir(),
        )
    }

    fn data_flag(&self) -> Option<String> {
        self.flag("--data-dir", &self.data, &crate::utils::safe_data_dir())
    }

    /// The flags as they sit in a command line, each preceded by its space: `" --config-dir X"`.
    /// Empty when the run used the defaults, so the common task is byte-for-byte what it was.
    fn flag_spellings(&self) -> String {
        let mut out = String::new();
        for flag in [self.config_flag(), self.data_flag()].into_iter().flatten() {
            out.push(' ');
            out.push_str(&flag);
        }
        out
    }

    /// The same two flags as argv elements, for a renderer whose job is an array — launchd hands
    /// `ProgramArguments` to `execve` directly, so a quoted string would arrive with its quotes.
    fn as_argv(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (which, root, default) in [
            (
                "--config-dir",
                &self.config,
                crate::utils::safe_config_dir(),
            ),
            ("--data-dir", &self.data, crate::utils::safe_data_dir()),
        ] {
            if root != &default && !root.as_os_str().is_empty() {
                out.push(which.to_string());
                out.push(root.display().to_string());
            }
        }
        out
    }
}

/// One schedule the machine holds **that Shall itself put there**, with the command line a
/// firing would run. The declaring run's identity is written into that line — the binary and,
/// since `c7dd0b67`, the `--config-dir`/`--data-dir` roots it was declared from (#192) — and a
/// sweep that must not delete another config's schedules reads that line to tell "mine,
/// retracted" from "somebody else's" (#182).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldTask {
    pub name: String,
    /// The argv as a firing would exec it, first element the binary, split with the same
    /// `schedule::words` the renderers write with. Empty when the scheduler answered with a
    /// name but no command line.
    pub argv: Vec<String>,
    /// Whether the scheduler's own store certifies *Shall* wrote this task, independently of
    /// the command line. systemd and launchd can (the file honours Shall's prefix); Task
    /// Scheduler cannot, because a listing by prefix is all it ever has (#182).
    pub certified: bool,
}

/// Whose a held schedule is, read from the command line it carries.
///
/// **`Ours` is the only answer the sweep may remove on** (#182). A task that names roots this
/// run did not use was declared by another config; a task with no roots recorded was written
/// before `c7dd0b67`, and the only thing about it that still identifies its author is the
/// binary it fires — if that is not this binary, it is not this run's task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleOrigin {
    /// This run's config declared it (every recorded root equals this run's), or it carries no
    /// roots and would fire this very binary — the legacy task written before `c7dd0b67` — or
    /// the store certifies authorship and there was no command line to read (the C34 ruling's
    /// own words, and the `#173` pin's shape).
    Ours,
    /// The line names a root this run did not use, so another config declared it.
    AnotherConfig {
        config: Option<String>,
        data: Option<String>,
    },
    /// No roots recorded, and the binary the task would fire is not this one.
    ForeignBinary { binary: String },
    /// The line could not be read back (including a flag left dangling, which a renderer of
    /// ours can never write).
    Unreadable,
}

/// Decide whose a held schedule is, from its argv, the store's word on authorship, and this
/// run's roots.
///
/// A task registered since `c7dd0b67` carries the roots it was declared from as the flags
/// immediately after the binary. Compare them to this run's roots; a root with no flag means
/// the declaring run used that root's OS default (`Roots::flag` omits a flag exactly then), so
/// it belongs here only if this run used that root's default too. A task with **no** flags at
/// all — every task written before that commit, and Windows tasks written by the gap #182
/// closes — has one surviving token of its author, the binary in its argv: the C34 ruling
/// keeps sweeping it (so `#158` keeps working for upgraders) only when that binary is this
/// one. A line that could not be read at all is taken on the store's own word (`certified`) or
/// left alone — see `HeldTask::certified` for who can give that word.
///
/// `pub` for the tests' sake, like `units_in`.
pub fn schedule_origin(
    argv: &[String],
    certified: bool,
    roots: &Roots,
    shall_bin: &Path,
) -> ScheduleOrigin {
    let mut config: Option<&String> = None;
    let mut data: Option<&String> = None;
    let mut malformed = false;
    {
        // The renderer writes the flags in argv positions 1..; the first non-flag token is the
        // command, and everything after it is the command's own business.
        let mut it = argv.iter().skip(1);
        while let Some(arg) = it.next() {
            match arg.as_str() {
                "--config-dir" => match it.next() {
                    Some(v) => config = Some(v),
                    None => malformed = true,
                },
                "--data-dir" => match it.next() {
                    Some(v) => data = Some(v),
                    None => malformed = true,
                },
                _ => break,
            }
        }
    }
    if malformed {
        return ScheduleOrigin::Unreadable;
    }
    if config.is_some() || data.is_some() {
        // A recorded root must equal this run's; a root with no flag is the default, and so
        // matches only a run that used the default.
        let cfg_ok = match config {
            Some(v) => Path::new(v.as_str()) == roots.config.as_path(),
            None => roots.config == crate::utils::safe_config_dir(),
        };
        let data_ok = match data {
            Some(v) => Path::new(v.as_str()) == roots.data.as_path(),
            None => roots.data == crate::utils::safe_data_dir(),
        };
        if cfg_ok && data_ok {
            ScheduleOrigin::Ours
        } else {
            ScheduleOrigin::AnotherConfig {
                config: config.cloned(),
                data: data.cloned(),
            }
        }
    } else if argv.is_empty() {
        // Nothing to read, so the flag and binary gates are both unanswered. A store that can
        // certify authorship decides here — systemd's unit file and launchd's plist with
        // Shall's prefix in them say yes, and that is the gate `#158` (and the #173 pin) stood
        // on. A store with nothing left to ask (Task Scheduler's `/XML` query failed) does not
        // get to call an unresid task ours.
        if certified {
            ScheduleOrigin::Ours
        } else {
            ScheduleOrigin::Unreadable
        }
    } else {
        match argv.first() {
            Some(bin) if Path::new(bin) == shall_bin => ScheduleOrigin::Ours,
            Some(bin) => ScheduleOrigin::ForeignBinary {
                binary: bin.clone(),
            },
            None => ScheduleOrigin::Unreadable,
        }
    }
}

impl TaskTarget {
    pub fn resolve(user: Option<&str>) -> Result<Self> {
        Ok(Self {
            account: match user {
                None => None,
                Some(name) => Some(crate::core::account::Account::resolve(name)?),
            },
            roots: Roots::default(),
        })
    }

    /// The roots of the run that declared the task, for when this target is provisioned.
    pub fn with_roots(mut self, roots: Roots) -> Self {
        self.roots = roots;
        self
    }

    pub fn account(&self) -> Result<crate::core::account::Account> {
        match &self.account {
            Some(account) => Ok(account.clone()),
            None => crate::core::account::Account::current(),
        }
    }

    /// Where this target's schedules append their output.
    ///
    /// **A scoped target logs into its own account's data directory; an unscoped one keeps the
    /// path it has always used.** That is not the same value: `safe_data_dir` is a function of
    /// the environment (`XDG_DATA_HOME` moves it) and of the platform (`dirs` answers
    /// `~/Library/Application Support` on macOS), while an account's data directory is its home
    /// plus a convention. One function rather than the copy each provisioner carried, because the
    /// two copies disagreed on exactly the case that mattered — a macOS unscoped schedule wrote
    /// to a shall-only `.local/share` tree that no `dirs` caller would ever look in.
    pub fn log_path(&self) -> PathBuf {
        let dir = match &self.account {
            Some(account) => account
                .data_dir()
                .unwrap_or_else(|_| crate::utils::safe_data_dir()),
            None => crate::utils::safe_data_dir(),
        };
        dir.join("schedule.log")
    }

    pub fn is_scoped(&self) -> bool {
        self.account.is_some()
    }

    pub fn can_register_for_another_account() -> bool {
        !cfg!(windows)
    }

    pub fn refuse_no_user_domain(&self, what: &str, platform: &str, reason: &str) -> Error {
        let Some(account) = &self.account else {
            return Error::Unsupported(format!(
                "this platform has no scheduler Shall can drive ({platform})"
            ));
        };
        Error::Refused(format!(
            "`{what}` is declared for the account `{}`, and Shall cannot reach a scheduler for \
             another account here: {reason}. Nothing was written for `{}`, and Shall will not \
             fall back to this account's scheduler — a task placed in the wrong account's \
             domain reports as scheduled and runs as the wrong person.",
            account.name(),
            account.name(),
        ))
    }
}

#[async_trait]
pub trait TaskProvisioner: Send + Sync {
    async fn add_task(
        &self,
        executor: &CommandExecutor,
        config: &ScheduleConfig,
        shall_path: &Path,
        target: &TaskTarget,
    ) -> Result<()>;
    async fn remove_task(
        &self,
        executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> Result<()>;
    async fn is_task_active(
        &self,
        executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> bool;

    /// Refuse, by name, an option this scheduler cannot express.
    ///
    /// The alternative is accepting the option and dropping it, which is the same failure as a
    /// cron silently widened into DAILY: the declaration says one thing, the machine does
    /// another, and both report success. `jitter` dropped on 500 machines is 500 machines
    /// hitting the mirror on the same second.
    fn refuse_unsupported(&self, config: &ScheduleConfig) -> std::result::Result<(), String>;

    /// What Shall would provision for `config`, in this scheduler's terms.
    fn rendered(
        &self,
        config: &ScheduleConfig,
        shall_bin: &Path,
        target: &TaskTarget,
    ) -> std::result::Result<Provisioned, String>;

    async fn read_task(
        &self,
        executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> Reading;

    /// The schedules this scheduler holds **that Shall itself put there**, by declared name.
    ///
    /// Every other question here starts from a declaration and asks the machine about it. This
    /// is the reverse question, and it exists because a unit can outlive its declaration: the
    /// owner's `shall-nightly` timer kept firing at 03:00 for a week after the `schedules` file
    /// that created it was gone, and nothing in the program could see it — `sync` converges what
    /// the files say and has no way to notice what they stopped saying (#158).
    ///
    /// **`None` is not `Some(vec![])`.** A scheduler that could not be asked must not be read
    /// as "this machine holds nothing", or a converge would walk away from units it simply
    /// could not see — `Reading::Unreadable`'s rule, one question over.
    ///
    /// **The prefix is the safety argument, not the directory.** `shall-`, `com.shall.` and
    /// `Shall_` are what Shall writes, and only a name carrying one of them may ever be removed
    /// by this route: a task a person wrote by hand is not a candidate however similar its name
    /// looks.
    async fn installed(
        &self,
        executor: &CommandExecutor,
        target: &TaskTarget,
    ) -> Option<Vec<String>>;

    /// [`installed`](Self::installed) with each name's command line, for the sweep that must
    /// not remove another config's tasks (#182). A schedule's declaring run is read back from
    /// the line it would fire (`schedule_origin`), because nothing else survives the process
    /// that wrote it — and names alone cannot tell "mine, retracted" from "somebody else's".
    ///
    /// The default answers a scheduler that records no command line: names with an empty argv,
    /// which the sweep must then treat as unreadable and leave alone. The real provisioners
    /// read their stores back and override.
    async fn installed_with_command(
        &self,
        executor: &CommandExecutor,
        target: &TaskTarget,
    ) -> Option<Vec<HeldTask>> {
        self.installed(executor, target).await.map(|names| {
            names
                .into_iter()
                .map(|name| HeldTask {
                    name,
                    argv: Vec::new(),
                    certified: false,
                })
                .collect()
        })
    }
}

pub struct SchedulerManager {
    provisioner: std::sync::Arc<dyn TaskProvisioner>,
    shall_bin_path: PathBuf,
    /// The roots of the run that is declaring schedules. **Everything a task fires with has to be
    /// resolvable later, and a path is the only thing that survives the process that wrote it**
    /// (#192): `shall --config-dir X sync` used to register `X`'s schedules and then have them
    /// fire `shall sync` against the default config, which is a different machine description.
    roots: Roots,
}

impl SchedulerManager {
    pub fn new() -> Result<Self> {
        debug!("Detecting system-native task runner.");
        Self::with_provisioner(std::sync::Arc::from(Self::detect()?))
    }

    pub fn with_provisioner(provisioner: std::sync::Arc<dyn TaskProvisioner>) -> Result<Self> {
        let shall_bin_path = std::env::current_exe()
            .map_err(|e| Error::Io(format!("Failed to locate current Shall binary: {}", e)))?;
        Ok(Self {
            provisioner,
            shall_bin_path,
            roots: Roots::default(),
        })
    }

    /// A manager that writes the roots of `config` into every task it registers.
    pub fn for_config(config: &crate::config::Config) -> Result<Self> {
        Ok(Self::new()?.with_roots(Roots::of(config)))
    }

    /// The same manager, told which roots the declaring run used.
    pub fn with_roots(mut self, roots: Roots) -> Self {
        self.roots = roots;
        self
    }

    pub fn detect() -> Result<Box<dyn TaskProvisioner>> {
        if cfg!(target_os = "linux") {
            Ok(Box::new(LinuxSystemdProvisioner))
        } else if cfg!(target_os = "macos") {
            Ok(Box::new(MacLaunchdProvisioner))
        } else if cfg!(target_os = "windows") {
            Ok(Box::new(WindowsTaskProvisioner))
        } else {
            Err(Error::UnsupportedPlatform(
                "Native scheduling is not supported on this OS variant.".into(),
            ))
        }
    }

    /// Register `cfg` with the OS scheduler (systemd/launchd/Task Scheduler) — the declarative
    /// path (S21). Unlike [`add_schedule`], it does NOT write to `preferences.toml`: a `schedule:`
    /// declared in the model lives in the `schedules` file, and `sync` provisions it from
    /// there on every run. Idempotent by nature — `add_task` re-registers the same task.
    pub async fn provision(&self, executor: &CommandExecutor, cfg: &ScheduleConfig) -> Result<()> {
        Self::validate_cron(&cfg.name, &cfg.cron)?;
        // Before anything is written, and for every provisioner, because an option accepted and
        // dropped is a declaration the machine does not keep.
        self.provisioner
            .refuse_unsupported(cfg)
            .map_err(|e| Error::Refused(format!("`schedule:{}`: {}", cfg.name, e)))?;
        let target = TaskTarget::resolve(cfg.user.as_deref())?.with_roots(self.roots.clone());
        self.provisioner
            .add_task(executor, cfg, &self.shall_bin_path, &target)
            .await
    }

    /// Remove a task from the OS scheduler by name, without touching preferences.toml — the undo
    /// side of [`provision`], used when a `schedule:` line is deleted (S20 drift).
    /// `reaped` is proof the removal guard was consulted — see
    /// [`Reaped`](crate::app::sync::guard::Reaped).
    pub async fn deprovision(
        &self,
        executor: &CommandExecutor,
        name: &str,
        user: Option<&str>,
        _reaped: crate::app::sync::guard::Reaped,
    ) -> Result<()> {
        let target = TaskTarget::resolve(user)?;
        self.provisioner.remove_task(executor, name, &target).await
    }

    /// Every schedule this machine holds that Shall put there, by declared name.
    ///
    /// `None` when the scheduler could not be asked. The caller must treat that as "unknown",
    /// never as "nothing" — see [`TaskProvisioner::installed`].
    pub async fn installed(
        &self,
        executor: &CommandExecutor,
        user: Option<&str>,
    ) -> Option<Vec<String>> {
        let target = TaskTarget::resolve(user).ok()?;
        self.provisioner.installed(executor, &target).await
    }

    /// Every schedule this machine holds that Shall put there, with the command line each
    /// would fire, for the sweep that must not remove another config's tasks (#182).
    ///
    /// `None` when the scheduler could not be asked — see [`TaskProvisioner::installed`].
    pub async fn installed_tasks(
        &self,
        executor: &CommandExecutor,
        user: Option<&str>,
    ) -> Option<Vec<HeldTask>> {
        let target = TaskTarget::resolve(user).ok()?;
        self.provisioner
            .installed_with_command(executor, &target)
            .await
    }

    /// Whose the held schedule is, asked against this run's roots and this binary (#182):
    /// `Ours` is the only answer the sweep may remove on.
    pub fn origin_of(&self, task: &HeldTask) -> ScheduleOrigin {
        schedule_origin(
            &task.argv,
            task.certified,
            &self.roots,
            &self.shall_bin_path,
        )
    }

    /// How this machine stands against one declaration — the read-back half of `schedule:`.
    ///
    /// Provisioning has always been idempotent, so the machine converged whatever the reporting
    /// said; what it could not do was *say* it had changed anything. `@cron=` and `@run=` are
    /// not in the ledger key, so an edited schedule is the same key, is found in the applied
    /// ledger, and was reported as nothing to do while `sync` rewrote it underneath — `J2`'s
    /// defect on a different kind.
    ///
    /// **The key is deliberately not widened to carry them.** A `setting:`'s scope makes two
    /// different subjects, so `J2`'s fix worked there; a schedule's name *is* its identity at the
    /// OS scheduler, so `schedule:nightly@cron=old` and `schedule:nightly@cron=new` are one cron
    /// entry — `reconcile` would deprovision by name the entry the apply phase had just written,
    /// and editing a schedule would silently delete it. Reading the machine answers the question
    /// the key cannot.
    pub async fn standing(&self, executor: &CommandExecutor, cfg: &ScheduleConfig) -> Standing {
        // A declaration this scheduler will refuse is not in effect and never will be. Reported
        // as work rather than as unverifiable, so `check` is red about it and the refusal that
        // follows names the option.
        if self.provisioner.refuse_unsupported(cfg).is_err() {
            return Standing::Missing;
        }
        let Ok(target) = TaskTarget::resolve(cfg.user.as_deref()) else {
            return Standing::Unknown;
        };
        let Ok(want) = self
            .provisioner
            .rendered(cfg, &self.shall_bin_path, &target)
        else {
            return Standing::Missing;
        };
        match self
            .provisioner
            .read_task(executor, &cfg.name, &target)
            .await
        {
            Reading::Unreadable => Standing::Unknown,
            Reading::Absent => Standing::Missing,
            Reading::Holds(got) => {
                if got.spec != want.spec {
                    Standing::Differs("what it runs, or when it runs, is not what is declared")
                } else if got.armed != want.armed {
                    Standing::Differs(if want.armed {
                        "it is registered but will not fire"
                    } else {
                        "it is registered and armed, and the declaration says it should not fire"
                    })
                } else {
                    Standing::Matches
                }
            }
        }
    }

    /// Reject an invalid cron before it reaches the OS scheduler. One implementation, in the
    /// model, so a bad cron is the same error whether it came from a file or a flag.
    fn validate_cron(name: &str, cron: &str) -> Result<()> {
        crate::model::schedule::validate_cron(cron).map_err(|e| {
            Error::Validation(format!(
                "Invalid cron syntax for task '{}': {}. Rejection issued.",
                name, e
            ))
        })
    }
}

// The XML text helpers, shared because two of the three dialects speak XML — launchd's plist
// and Task Scheduler's task document — and one escaping table is one table. A copy each is how
// an escape gets fixed in one dialect and left broken in the other.

/// The three characters XML cares about, escaped for a text node.
///
/// Text in a plist is not markup, so `&`, `<` and `>` are the whole list: a schedule named
/// `a & b` or a path with a `<` in it would otherwise make the plist unparseable, and launchd
/// answers an unparseable plist by not running the job at all.
pub(crate) fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Is this schedule meant to fire? An option nobody wrote leaves the schedule armed, which is
/// what every schedule did before the option existed.
pub(crate) fn armed(config: &ScheduleConfig) -> bool {
    config.enabled.unwrap_or(true)
}

/// The whole `<tag> … </tag>` element, angle brackets included, so a nested lookup can be
/// scoped to it. `<Settings><Enabled>` and a trigger's own `<Enabled>` are different questions,
/// and a document-wide search for the first `<Enabled>` answers whichever comes first.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn xml_element<'a>(doc: &'a str, tag: &str) -> Option<&'a str> {
    let close = format!("</{}>", tag);
    // The opening tag may carry attributes (`<BootTrigger id="Resume">`), so it is matched on
    // its prefix and the `>` is found after it.
    let open = format!("<{}", tag);
    let at = doc.find(&open)?;
    let after_name = doc[at + open.len()..].chars().next()?;
    if after_name != '>' && !after_name.is_whitespace() {
        return None;
    }
    let end = doc[at..].find(&close)? + at + close.len();
    Some(&doc[at..end])
}

/// The five entities [`xml_escape`] and Task Scheduler itself can produce, back as characters.
pub(crate) fn unescape_xml(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

// One subject per file: this vocabulary, shared by every scheduler, and one module per scheduler
// Shall can drive. The public names are re-exported, so `app::scheduler::*` is still the whole
// surface — the tests import that way, and a split that moved a name would be a rename.
mod launchd;
mod schtasks;
mod systemd;

pub use launchd::*;
pub use schtasks::*;
pub use systemd::*;
