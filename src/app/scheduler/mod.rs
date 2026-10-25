use crate::config::config::ScheduleConfig;
use crate::core::executor::StdOutput;
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
pub fn rotate_log_if_large() {
    const MAX_BYTES: u64 = 10 * 1024 * 1024;
    let log = crate::utils::safe_data_dir().join("schedule.log");
    let Ok(meta) = std::fs::metadata(&log) else {
        return;
    };
    if meta.len() <= MAX_BYTES {
        return;
    }
    let rotated = log.with_extension("log.1");
    let _ = std::fs::remove_file(&rotated);
    match std::fs::rename(&log, &rotated) {
        Ok(()) => eprintln!(
            "schedule.log passed 10 MiB and was set aside as schedule.log.1 (the old .1 is gone)"
        ),
        Err(e) => eprintln!("could not rotate schedule.log: {e}"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskTarget {
    pub account: Option<crate::core::account::Account>,
}

impl TaskTarget {
    pub fn resolve(user: Option<&str>) -> Result<Self> {
        Ok(Self {
            account: match user {
                None => None,
                Some(name) => Some(crate::core::account::Account::resolve(name)?),
            },
        })
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
}

pub struct SchedulerManager {
    provisioner: std::sync::Arc<dyn TaskProvisioner>,
    shall_bin_path: PathBuf,
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
        })
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
        let target = TaskTarget::resolve(cfg.user.as_deref())?;
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

/// Is this schedule meant to fire? An option nobody wrote leaves the schedule armed, which is
/// what every schedule did before the option existed.
fn armed(config: &ScheduleConfig) -> bool {
    config.enabled.unwrap_or(true)
}

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub struct LinuxSystemdProvisioner;

impl LinuxSystemdProvisioner {
    fn unit_stem(name: &str) -> String {
        format!("shall-{}", name)
    }

    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
    /// crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn unit_dir(target: &TaskTarget) -> Option<PathBuf> {
        match &target.account {
            Some(account) => account.systemd_user_dir().ok(),
            None => Some(dirs::config_dir()?.join("systemd").join("user")),
        }
    }

    /// The `.service` unit, for both shapes a schedule can take.
    ///
    /// One renderer, because there were two and they disagreed: the boot shape was written by
    /// overwriting the file the ordinary shape had just written, and the replacement dropped
    /// `StandardOutput=`/`StandardError=` — so an `@reboot` job's output went nowhere while
    /// every other job's was appended to `schedule.log`.
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
    /// crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn service_unit(config: &ScheduleConfig, shall_bin: &Path, target: &TaskTarget) -> String {
        let log = target.log_path();
        let mut unit = format!(
            "[Unit]\nDescription=Shall {kind}: {name}\n\n\
             [Service]\nType=oneshot\nExecStart={bin} {cmd}\n\
             StandardOutput=append:{log}\nStandardError=append:{log}\n",
            kind = if config.cron == "@reboot" {
                "Reboot Job"
            } else {
                "Job"
            },
            name = config.name,
            bin = shall_bin.display(),
            cmd = config.command,
            log = log.display(),
        );
        if config.cron == "@reboot" {
            unit.push_str("\n[Install]\nWantedBy=default.target\n");
        }
        unit
    }

    /// The `.timer` unit. `@reboot` has none — it is an `[Install]` on the service.
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
    /// crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn timer_unit(&self, config: &ScheduleConfig) -> String {
        let mut unit = format!(
            "[Unit]\nDescription=Shall Schedule Timer for {name}\n\n\
             [Timer]\nOnCalendar={calendar}\nPersistent={persistent}\n",
            name = config.name,
            calendar = self.map_cron_to_systemd(&config.cron),
            // Undeclared is `true`, which is what the timer always said before the option
            // existed, so adding it moved nobody's schedule.
            persistent = config.persistent.unwrap_or(true),
        );
        if let Some(seconds) = config.jitter {
            unit.push_str(&format!("RandomizedDelaySec={}\n", seconds));
        }
        unit.push_str("\n[Install]\nWantedBy=timers.target\n");
        unit
    }

    async fn systemctl_says(
        &self,
        executor: &CommandExecutor,
        verb: &str,
        unit: &str,
        expect: &str,
        target: &TaskTarget,
    ) -> bool {
        let args = ["--no-pager", "--user", verb, unit];
        let out = match &target.account {
            Some(account) => executor.run_output_as(account, "systemctl", &args).await,
            None => executor.run_output("systemctl", &args, false).await,
        };
        match out {
            Ok(text) => crate::utils::text::sanitize(&text) == expect,
            Err(_) => false,
        }
    }

    async fn systemctl_run(
        &self,
        executor: &CommandExecutor,
        target: &TaskTarget,
        args: &[&str],
    ) -> Result<StdOutput> {
        match &target.account {
            Some(account) => executor.run_as(account, "systemctl", args, false).await,
            None => executor.run("systemctl", args, false).await,
        }
    }

    async fn user_manager_is_reachable(
        &self,
        executor: &CommandExecutor,
        target: &TaskTarget,
    ) -> Result<()> {
        let account = match &target.account {
            Some(account) => account,
            None => return Ok(()),
        };
        let args = ["--no-pager", "--user", "daemon-reload"];
        executor
            .run_output_as(account, "systemctl", &args)
            .await
            .map(|_| ())
            .map_err(|e| {
                Error::Refused(format!(
                    "`schedule:{}` is declared for the account `{}`, and there is no user manager \
                     running for it — `systemctl --user` cannot connect to a bus for that account \
                     ({e}). Nothing was written. A user unit fires only while its account has a \
                     session, so Shall will not write one and report it as scheduled.",
                    account.name(),
                    account.name()
                ))
            })
    }

    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
    /// crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn map_cron_to_systemd(&self, cron: &str) -> String {
        match cron {
            "@hourly" => "hourly".into(),
            "@daily" => "daily".into(),
            "@weekly" => "weekly".into(),
            "@monthly" => "monthly".into(),
            "@yearly" | "@annually" => "yearly".into(),
            other => {
                let parts: Vec<&str> = other.split_whitespace().collect();
                if parts.len() < 5 {
                    return "daily".into();
                }

                // systemd OnCalendar = [DOW ]YYYY-MM-DD HH:MM:SS with `*` wildcards and
                // zero-padded time. Standard cron order is min hour dom month dow.
                let min = self.pad2(&self.translate_field(parts[0]));
                let hour = self.pad2(&self.translate_field(parts[1]));
                let dom = self.translate_field(parts[2]);
                let mon = self.translate_field(parts[3]);
                // The weekday field is mapped by TOKEN, not by character: a blind
                // `.replace('0', "Sun")` turned the range `10-12` into `1Sun-1Tue`. Shapes
                // that cannot be said are refused in `refuse_unsupported` before anything
                // is written.
                let dow = if parts[4] == "*" {
                    "*".to_string()
                } else {
                    map_vixie_dow_to_names(parts[4])
                };

                let date = format!("*-{}-{}", mon, dom);
                let time = format!("{}:{}:00", hour, min);

                if dow == "*" {
                    format!("{} {}", date, time)
                } else {
                    format!("{} {} {}", dow, date, time)
                }
            }
        }
    }

    /// Zero-pad a single-digit numeric field to two digits (systemd time); leave
    /// wildcards / ranges / step expressions untouched.
    fn pad2(&self, s: &str) -> String {
        if s.len() == 1 && s.chars().all(|c| c.is_ascii_digit()) {
            format!("0{}", s)
        } else {
            s.to_string()
        }
    }

    fn translate_field(&self, field: &str) -> String {
        if field == "*" {
            return "*".into();
        }
        if let Some(step) = field.strip_prefix("*/") {
            return format!("0/{}", step);
        }
        if field.contains('-') {
            return field.replace('-', "..");
        }
        field.to_string()
    }
}

#[async_trait]
impl TaskProvisioner for LinuxSystemdProvisioner {
    fn refuse_unsupported(&self, config: &ScheduleConfig) -> std::result::Result<(), String> {
        // A user timer runs as the user who owns the session, and there is no `--user` unit that
        // can raise that. Making one would mean writing into /etc/systemd/system as root, which
        // is a different schedule with a different lifetime and a different removal.
        if config.elevated == Some(true) {
            return Err(
                "`elevated = true` needs a system unit, and Shall's schedules are user \
                        timers — a `--user` unit runs as you and cannot raise itself. Run the \
                        privileged part from the command the schedule invokes."
                    .into(),
            );
        }
        // `jitter` on a `@reboot` job has nowhere to live: the delay is a `[Timer]` setting and
        // a boot job is an `[Install]` on the service.
        if config.jitter.is_some() && config.cron == "@reboot" {
            return Err(
                "`jitter` is a timer setting and `@reboot` has no timer — it is an \
                        install on the service. Delay the work inside the command instead."
                    .into(),
            );
        }
        if config.persistent.is_some() && config.cron == "@reboot" {
            return Err(
                "`persistent` asks a timer to catch up a missed firing, and `@reboot` \
                        has no timer — it fires when the machine comes up, which is the same \
                        thing."
                    .into(),
            );
        }
        // Two shapes of silent lie, refused by name rather than translated wrong.
        if !matches!(
            config.cron.as_str(),
            "@hourly" | "@daily" | "@weekly" | "@monthly" | "@yearly" | "@annually"
        ) {
            let parts: Vec<&str> = config.cron.split_whitespace().collect();
            if parts.len() == 5 {
                let (dom, dow) = (parts[2], parts[4]);
                // Vixie cron fires when EITHER a restricted dom or a restricted dow
                // matches; OnCalendar requires BOTH. Translating one into the other
                // narrowed the schedule without saying so — the exact class the Windows
                // mapper refuses by name.
                if dom != "*" && dow != "*" {
                    return Err(format!(
                        "`{}` restricts both day-of-month and day-of-week. Vixie cron \
                         fires on either; systemd's OnCalendar requires both. Split it \
                         into two `schedule:` lines.",
                        config.cron
                    ));
                }
                // OnCalendar's weekday field takes day names and ranges of them — a step
                // (`*/2`, `0/2`) dies at unit start, AFTER sync reported success.
                if dow.contains('/') {
                    return Err(format!(
                        "day-of-week step `{dow}` has no systemd equivalent: \
                         OnCalendar's weekday field takes day names and ranges, not steps.",
                        dow = dow
                    ));
                }
            }
        }
        Ok(())
    }

    fn rendered(
        &self,
        config: &ScheduleConfig,
        shall_bin: &Path,
        target: &TaskTarget,
    ) -> std::result::Result<Provisioned, String> {
        let mut spec = Self::service_unit(config, shall_bin, target);
        if config.cron != "@reboot" {
            spec.push_str(UNIT_SEPARATOR);
            spec.push_str(&self.timer_unit(config));
        }
        Ok(Provisioned {
            spec,
            armed: armed(config),
        })
    }

    async fn read_task(
        &self,
        executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> Reading {
        let Some(dir) = Self::unit_dir(target) else {
            return Reading::Unreadable;
        };
        let stem = Self::unit_stem(name);
        let mut spec = match std::fs::read_to_string(dir.join(format!("{}.service", stem))) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Reading::Absent,
            // A unit directory this account cannot read is not an empty one.
            Err(_) => return Reading::Unreadable,
        };
        match std::fs::read_to_string(dir.join(format!("{}.timer", stem))) {
            Ok(text) => {
                spec.push_str(UNIT_SEPARATOR);
                spec.push_str(&text);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Reading::Unreadable,
        }
        Reading::Holds(Provisioned {
            spec,
            armed: self.is_task_active(executor, name, target).await,
        })
    }

    async fn add_task(
        &self,
        executor: &CommandExecutor,
        config: &ScheduleConfig,
        shall_bin: &Path,
        target: &TaskTarget,
    ) -> Result<()> {
        self.user_manager_is_reachable(executor, target).await?;
        let systemd_dir = Self::unit_dir(target).ok_or_else(|| {
            target.refuse_no_user_domain(
                &format!("schedule:{}", config.name),
                "systemd",
                "the account's home directory is not resolvable, so there is no per-user unit \
                 directory to write into",
            )
        })?;

        crate::utils::file::ensure_dir(&systemd_dir)?;

        let stem = Self::unit_stem(&config.name);
        let service_unit = format!("{}.service", stem);
        let timer_unit = format!("{}.timer", stem);
        let boot_job = config.cron == "@reboot";
        let arm = armed(config);

        executor
            .write_atomic(
                &systemd_dir.join(&service_unit),
                &Self::service_unit(config, shall_bin, target),
            )
            .await?;
        if !boot_job {
            executor
                .write_atomic(&systemd_dir.join(&timer_unit), &self.timer_unit(config))
                .await?;
        }

        self.systemctl_run(executor, target, &["--no-pager", "--user", "daemon-reload"])
            .await?;

        // The unit that carries the schedule: the timer, or the service itself for a boot job.
        let carrier = if boot_job { &service_unit } else { &timer_unit };
        if arm {
            let mut args = vec!["--no-pager", "--user", "enable"];
            // `--now` starts a timer; there is nothing to start about a boot job, and asking
            // systemd to start a oneshot service here would run the command immediately.
            if !boot_job {
                args.push("--now");
            }
            args.push(carrier);
            self.systemctl_run(executor, target, &args).await?;
        } else {
            // Declared and deliberately silent. `disable --now` rather than "do not enable",
            // because the previous sync may have armed it and a declaration that changed has to
            // reach the machine.
            let _ = self
                .systemctl_run(
                    executor,
                    target,
                    &["--no-pager", "--user", "disable", "--now", carrier],
                )
                .await;
            if self.is_task_active(executor, &config.name, target).await {
                return Err(Error::Io(format!(
                    "`schedule:{}` is declared `enabled = false` and its systemd unit is still \
                     active. Check `systemctl --user status {}`.",
                    config.name, carrier
                )));
            }
        }

        Ok(())
    }

    async fn remove_task(
        &self,
        executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> Result<()> {
        let stem = Self::unit_stem(name);
        let timer_name = format!("{}.timer", stem);
        let service_name = format!("{}.service", stem);

        let _ = self
            .systemctl_run(
                executor,
                target,
                &["--no-pager", "--user", "disable", "--now", &timer_name],
            )
            .await;
        let _ = self
            .systemctl_run(
                executor,
                target,
                &["--no-pager", "--user", "disable", "--now", &service_name],
            )
            .await;

        if let Some(systemd_dir) = Self::unit_dir(target) {
            remove_generated(&systemd_dir.join(&timer_name))?;
            remove_generated(&systemd_dir.join(&service_name))?;
        }
        // `disable` is allowed to fail — a unit that was never enabled reports failure — so
        // the end state is what gets asserted. A timer still running after this is a schedule
        // Shall would otherwise report as removed while it keeps firing.
        if self.is_task_active(executor, name, target).await {
            return Err(Error::Io(format!(
                "the systemd timer for `{}` is still active after removal. Check \
                 `systemctl --user status {}`.",
                name, timer_name
            )));
        }
        Ok(())
    }

    /// Does this schedule still fire?
    ///
    /// **Both shapes, because only one was asked about.** A timer job is `is-active` on the
    /// timer; a `@reboot` job has no timer at all, so the question is whether the service is
    /// enabled — and asking only the first meant the end-state assertion in `remove_task` was
    /// vacuous for every boot job, which is precisely the case where a surviving unit runs the
    /// command again on the next boot.
    async fn is_task_active(
        &self,
        executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> bool {
        let stem = Self::unit_stem(name);
        self.systemctl_says(
            executor,
            "is-active",
            &format!("{}.timer", stem),
            "active",
            target,
        )
        .await
            || self
                .systemctl_says(
                    executor,
                    "is-enabled",
                    &format!("{}.service", stem),
                    "enabled",
                    target,
                )
                .await
    }
}

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub struct MacLaunchdProvisioner;

/// Vixie day-of-week (`1-5`, `0,6`, `*`) to OnCalendar day names (`Mon..Fri`, `Sun,Sat`).
///
/// By token: a character-level replace turned `10` into `1Sun` and read ranges as glue.
/// 7 is Sunday in cron and out of the table's range, hence the mod.
fn map_vixie_dow_to_names(field: &str) -> String {
    const NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let name = |tok: &str| -> String {
        match tok.parse::<u32>() {
            Ok(n) => NAMES[(n % 7) as usize].to_string(),
            Err(_) => tok.to_string(),
        }
    };
    field
        .split(',')
        .map(|atom| match atom.split_once('-') {
            Some((a, b)) => format!("{}..{}", name(a), name(b)),
            None => name(atom),
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Whether every cron field is a single integer or `*` — the whole of what
/// `StartCalendarInterval` can say.
fn cron_is_launchd_expressible(cron: &str) -> bool {
    let parts: Vec<&str> = cron.split_whitespace().collect();
    if parts.len() != 5 {
        return false;
    }
    parts
        .iter()
        .all(|f| *f == "*" || (!f.contains([',', '-', '/', '*']) && f.parse::<u32>().is_ok()))
}

impl MacLaunchdProvisioner {
    fn label(name: &str) -> String {
        format!("com.shall.{}", name)
    }

    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
    /// crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn plist_path(name: &str, target: &TaskTarget) -> Option<PathBuf> {
        let dir = match &target.account {
            Some(account) => account.launch_agents_dir().ok()?,
            None => dirs::home_dir()?.join("Library").join("LaunchAgents"),
        };
        Some(dir.join(format!("{}.plist", Self::label(name))))
    }

    async fn launchctl(
        &self,
        executor: &CommandExecutor,
        target: &TaskTarget,
        args: &[&str],
    ) -> Result<StdOutput> {
        match &target.account {
            Some(account) => executor.run_as(account, "launchctl", args, false).await,
            None => executor.run("launchctl", args, false).await,
        }
    }

    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
    /// crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn plist(&self, config: &ScheduleConfig, shall_bin: &Path, target: &TaskTarget) -> String {
        let label = Self::label(&config.name);
        let schedule_xml = if config.cron == "@reboot" {
            "<key>RunAtLoad</key><true/>".to_string()
        } else {
            format!(
                "<key>StartCalendarInterval</key>{}",
                self.map_cron_to_launchd_xml(&config.cron)
            )
        };
        // launchd's own switch for a job that is present and must not fire. Written into the
        // file rather than left to `launchctl unload`, so a reboot does not quietly arm it.
        let disabled = if armed(config) {
            ""
        } else {
            "<key>Disabled</key><true/>\n"
        };

        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
            <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
            <plist version=\"1.0\">\n<dict>\n\
            <key>Label</key><string>{label}</string>\n\
            <key>ProgramArguments</key>\n<array>\n\
            <string>{bin}</string><string>{cmd}</string>\n</array>\n\
            {schedule}\n\
            {disabled}\
            <key>StandardOutPath</key><string>{log}</string>\n\
            <key>StandardErrorPath</key><string>{log}</string>\n\
            </dict>\n</plist>",
            label = label,
            bin = shall_bin.display(),
            cmd = config.command,
            schedule = schedule_xml,
            disabled = disabled,
            log = target.log_path().display()
        )
    }

    fn map_cron_to_launchd_xml(&self, cron: &str) -> String {
        let parts: Vec<&str> = cron.split_whitespace().collect();
        let (m, h, dom, mon, dow) = match cron {
            "@hourly" => ("0", "*", "*", "*", "*"),
            "@daily" => ("0", "0", "*", "*", "*"),
            "@weekly" => ("0", "0", "*", "*", "1"),
            "@monthly" => ("0", "0", "1", "*", "*"),
            _ if parts.len() >= 5 => (parts[0], parts[1], parts[2], parts[3], parts[4]),
            _ => ("0", "2", "*", "*", "*"),
        };

        let mut xml = String::from("<dict>");
        let keys = ["Minute", "Hour", "Day", "Month", "Weekday"];
        let vals = [m, h, dom, mon, dow];

        for (i, &val) in vals.iter().enumerate() {
            if val != "*" {
                let first_val = val.split([',', '-', '/']).next().unwrap_or("0");
                if let Ok(num) = first_val.parse::<u32>() {
                    xml.push_str(&format!("<key>{}</key><integer>{}</integer>", keys[i], num));
                }
            }
        }
        xml.push_str("</dict>");
        xml
    }
}

#[async_trait]
impl TaskProvisioner for MacLaunchdProvisioner {
    fn refuse_unsupported(&self, config: &ScheduleConfig) -> std::result::Result<(), String> {
        if config.jitter.is_some() {
            return Err(
                "`jitter` has no launchd equivalent — a `StartCalendarInterval` fires on \
                        the minute it names and nothing spreads it. Randomise inside the command \
                        the schedule invokes, or drop the option."
                    .into(),
            );
        }
        // `@reboot` is `RunAtLoad`, which fires when the agent loads and has no calendar to
        // miss — the same reason systemd refuses it on a boot job, said in launchd's terms.
        if config.persistent.is_some() && config.cron == "@reboot" {
            return Err(
                "`persistent` asks a calendar job to catch up a firing it slept through, \
                        and `@reboot` is `RunAtLoad` — it fires when the agent loads, which is \
                        the same thing."
                    .into(),
            );
        }
        // launchd runs a missed calendar job when the machine wakes, and offers no switch for
        // it. `true` is therefore what happens; `false` is a promise nothing here can keep.
        if config.persistent == Some(false) {
            return Err(
                "`persistent = false` has no launchd equivalent — a calendar job whose \
                        time passed while the machine was asleep runs on wake, and launchd \
                        offers no way to decline that."
                    .into(),
            );
        }
        if config.elevated == Some(true) {
            return Err(
                "`elevated = true` needs a LaunchDaemon, and Shall's schedules are \
                        LaunchAgents — an agent runs as you and cannot raise itself. Run the \
                        privileged part from the command the schedule invokes."
                    .into(),
            );
        }
        // **And expressibility, which was never checked.** `StartCalendarInterval` takes one
        // integer per field — no lists, no ranges, no steps. The old mapper kept the first
        // token of any of those and dropped the rest: `*/10 * * * *` parsed its step field
        // down to nothing, emitted an empty dict, and launchd fired the job EVERY MINUTE
        // while the read-back compared plist to plist and agreed for ever. A cron shape that
        // cannot be said in launchd's language is refused here, by name, before anything is
        // written.
        if config.cron != "@reboot" && !cron_is_launchd_expressible(&config.cron) {
            return Err(format!(
                "`{}` uses a cron list, range or step, and `StartCalendarInterval` takes one \
                 integer per field — there is no launchd equivalent. Name single values \
                 (e.g. `0 2 * * 1`) or several `schedule:` lines.",
                config.cron
            ));
        }
        Ok(())
    }

    fn rendered(
        &self,
        config: &ScheduleConfig,
        shall_bin: &Path,
        target: &TaskTarget,
    ) -> std::result::Result<Provisioned, String> {
        Ok(Provisioned {
            spec: self.plist(config, shall_bin, target),
            armed: armed(config),
        })
    }

    async fn read_task(
        &self,
        executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> Reading {
        let Some(path) = Self::plist_path(name, target) else {
            return Reading::Unreadable;
        };
        match std::fs::read_to_string(&path) {
            Ok(spec) => Reading::Holds(Provisioned {
                spec,
                armed: self.is_task_active(executor, name, target).await,
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Reading::Absent,
            Err(_) => Reading::Unreadable,
        }
    }

    async fn add_task(
        &self,
        executor: &CommandExecutor,
        config: &ScheduleConfig,
        shall_bin: &Path,
        target: &TaskTarget,
    ) -> Result<()> {
        let plist_path = Self::plist_path(&config.name, target).ok_or_else(|| {
            target.refuse_no_user_domain(
                &format!("schedule:{}", config.name),
                "launchd",
                "the account's home directory is not resolvable, so there is no per-user \
                 LaunchAgents directory to write into",
            )
        })?;

        if let Some(parent) = plist_path.parent() {
            crate::utils::file::ensure_dir(parent)?;
        }
        executor
            .write_atomic(&plist_path, &self.plist(config, shall_bin, target))
            .await?;

        if armed(config) {
            self.launchctl(executor, target, &["load", &plist_path.to_string_lossy()])
                .await?;
            return Ok(());
        }

        // Declared silent: the `Disabled` key keeps it that way across a reboot, and an unload
        // takes it out of this session. An agent that was never loaded reports failure, so the
        // end state is what gets asserted rather than the exit code.
        let _ = self
            .launchctl(executor, target, &["unload", &plist_path.to_string_lossy()])
            .await;
        if self.is_task_active(executor, &config.name, target).await {
            return Err(Error::Io(format!(
                "`schedule:{}` is declared `enabled = false` and its launchd agent is still \
                 loaded. Check `launchctl list {}`.",
                config.name,
                Self::label(&config.name)
            )));
        }
        Ok(())
    }

    async fn remove_task(
        &self,
        executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> Result<()> {
        if let Some(plist_path) = Self::plist_path(name, target) {
            let _ = self
                .launchctl(executor, target, &["unload", &plist_path.to_string_lossy()])
                .await;
            remove_generated(&plist_path)?;
        }
        // `unload` is allowed to fail (an agent that was never loaded reports failure); a job
        // still listed after this one is a schedule that keeps firing.
        if self.is_task_active(executor, name, target).await {
            return Err(Error::Io(format!(
                "the launchd agent for `{}` is still loaded after removal. Check \
                 `launchctl list {}`.",
                name,
                Self::label(name)
            )));
        }
        Ok(())
    }

    async fn is_task_active(
        &self,
        executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> bool {
        let args = ["list", &Self::label(name)];
        let out = match &target.account {
            Some(account) => executor.run_output_as(account, "launchctl", &args).await,
            None => executor.run_output("launchctl", &args, false).await,
        };
        out.is_ok()
    }
}

/// The five cron fields, with `@`-shorthands already expanded into them.
///
/// One expansion, shared. The shorthand table was written out once per provisioner, and the
/// Windows one simply did not have it — so `@daily` reached `split_whitespace()` as a single
/// field and came out the other side as the start time `02:@daily`. A table each is a table
/// that can be missing.
struct CronFields<'a> {
    minute: &'a str,
    hour: &'a str,
    dom: &'a str,
    month: &'a str,
    dow: &'a str,
}

/// `@weekly` is Monday here, not Sunday, because that is what the systemd and launchd mappings
/// have always done (`OnCalendar=weekly` is Mon 00:00). Matching vixie-cron instead would move
/// existing users' schedules by a day on two platforms to fix a third.
fn parse_cron(cron: &str) -> Option<CronFields<'_>> {
    let f = |minute, hour, dom, month, dow| {
        Some(CronFields {
            minute,
            hour,
            dom,
            month,
            dow,
        })
    };
    match cron.trim() {
        "@hourly" => f("0", "*", "*", "*", "*"),
        "@daily" | "@midnight" => f("0", "0", "*", "*", "*"),
        "@weekly" => f("0", "0", "*", "*", "1"),
        "@monthly" => f("0", "0", "1", "*", "*"),
        "@yearly" | "@annually" => f("0", "0", "1", "1", "*"),
        other => {
            let p: Vec<&str> = other.split_whitespace().collect();
            if p.len() < 5 {
                return None;
            }
            f(p[0], p[1], p[2], p[3], p[4])
        }
    }
}

/// `HH:mm`, which is the only start time Task Scheduler accepts.
///
/// The whole reported defect was `format!("{}:{}", hour, min)` on the raw cron fields: `0 3 * * *`
/// became `3:0` and `schtasks` answered `ERROR: Invalid starttime value.` A time is two digits
/// and two digits, always.
fn schtasks_time(hour: &str, minute: &str) -> Option<String> {
    let h: u8 = hour.parse().ok()?;
    let m: u8 = minute.parse().ok()?;
    if h > 23 || m > 59 {
        return None;
    }
    Some(format!("{:02}:{:02}", h, m))
}

/// Sunday-first day names, the order Task Scheduler's `<DaysOfWeek>` lists them in.
const DAY_NAMES: [&str; 7] = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
/// The `<DaysOfWeek>` element names, in the same order.
const DAY_ELEMENTS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const MONTH_NAMES: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];
/// The `<Months>` element names, in the same order.
const MONTH_ELEMENTS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// cron's day-of-week (0/7 = Sunday) as the day names `/D` takes, comma-separated.
fn schtasks_days(dow: &str) -> Option<String> {
    let name = |n: u8| DAY_NAMES.get((n % 7) as usize).copied();

    let mut out: Vec<&str> = Vec::new();
    for part in dow.split(',') {
        // `/D` takes a list and no ranges, so a range is expanded rather than passed through.
        if let Some((a, b)) = part.split_once('-') {
            let (a, b): (u8, u8) = (a.parse().ok()?, b.parse().ok()?);
            if a > b || b > 7 {
                return None;
            }
            for n in a..=b {
                out.push(name(n)?);
            }
        } else {
            out.push(name(part.parse().ok()?)?);
        }
    }
    if out.is_empty() {
        return None;
    }
    out.dedup();
    Some(out.join(","))
}

/// The `/SC …` arguments that make Task Scheduler do what a cron line says.
///
/// Windows has no cron, so this is a translation and not every sentence has one. Where a cron
/// cannot be expressed it is **refused by name** rather than widened into the nearest thing
/// Task Scheduler can do: the defect this replaces silently turned `0 3 * * 1` into a DAILY
/// task, which ran seven times as often as it was declared to and reported success each time.
/// A schedule that fires when it should not is worse than one that refuses to be created.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn map_cron_to_schtasks(cron: &str) -> std::result::Result<Vec<String>, String> {
    let s = |v: &str| v.to_string();
    let cannot = || {
        format!(
            "`{}` is a schedule Windows Task Scheduler cannot express. It understands a time of \
             day, a weekday, a day of the month, or a fixed interval — but not a combination of \
             an interval with a time window. Split it into separate schedules, or use a cron \
             this machine can keep.",
            cron
        )
    };

    if cron.trim() == "@reboot" {
        return Ok(vec![s("/SC"), s("ONSTART")]);
    }

    let c = parse_cron(cron).ok_or_else(|| {
        format!(
            "`{}` is not a cron expression: it needs five fields (minute hour day month weekday) \
             or one of @reboot, @hourly, @daily, @weekly, @monthly, @yearly.",
            cron
        )
    })?;

    // Sub-hourly. Task Scheduler's MINUTE/HOURLY intervals run around the clock, so they can
    // carry no other constraint — and pretending otherwise is how a schedule fires at times
    // nobody asked for.
    let unconstrained = c.dom == "*" && c.month == "*" && c.dow == "*";
    if c.minute == "*" || c.minute.starts_with("*/") {
        if !unconstrained || c.hour != "*" {
            return Err(cannot());
        }
        let mut args = vec![s("/SC"), s("MINUTE")];
        if let Some(step) = c.minute.strip_prefix("*/") {
            step.parse::<u16>().map_err(|_| cannot())?;
            args.extend([s("/MO"), s(step)]);
        }
        // Anchor the repetition: without /ST Task Scheduler starts counting from
        // registration time, invisible to the canonical comparison and to the user.
        let st = schtasks_time("0", "0").unwrap();
        args.extend([s("/ST"), st]);
        return Ok(args);
    }
    if c.hour == "*" || c.hour.starts_with("*/") {
        if !unconstrained {
            return Err(cannot());
        }
        let mut args = vec![s("/SC"), s("HOURLY")];
        if let Some(step) = c.hour.strip_prefix("*/") {
            step.parse::<u16>().map_err(|_| cannot())?;
            args.extend([s("/MO"), s(step)]);
        }
        // The first run of the hour, which is what the minute field means here.
        let st = schtasks_time("0", c.minute).ok_or_else(cannot)?;
        args.extend([s("/ST"), st]);
        return Ok(args);
    }

    let st = schtasks_time(c.hour, c.minute).ok_or_else(cannot)?;

    // A weekday beats a day of the month: `/SC WEEKLY` and `/SC MONTHLY` are exclusive, and a
    // cron naming both is the one shape with no Task Scheduler equivalent.
    if c.dow != "*" {
        if c.dom != "*" {
            return Err(cannot());
        }
        let days = schtasks_days(c.dow).ok_or_else(cannot)?;
        return Ok(vec![s("/SC"), s("WEEKLY"), s("/D"), days, s("/ST"), st]);
    }

    if c.dom != "*" || c.month != "*" {
        let mut args = vec![s("/SC"), s("MONTHLY")];
        if c.month != "*" {
            let n: usize = c.month.parse().map_err(|_| cannot())?;
            let name = MONTH_NAMES.get(n.wrapping_sub(1)).ok_or_else(cannot)?;
            args.extend([s("/M"), s(name)]);
        }
        // `/SC MONTHLY` with no `/D` is the 1st, which is also what a bare month means.
        let day = if c.dom == "*" { "1" } else { c.dom };
        day.parse::<u8>()
            .ok()
            .filter(|d| (1..=31).contains(d))
            .ok_or_else(cannot)?;
        args.extend([s("/D"), s(day), s("/ST"), st]);
        return Ok(args);
    }

    Ok(vec![s("/SC"), s("DAILY"), s("/ST"), st])
}

/// The value after `flag` in a `/SC …` argument list.
fn arg_after<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    let i = args.iter().position(|a| a == flag)?;
    args.get(i + 1).map(String::as_str)
}

/// Day names in Sunday-first order, so a cron that lists them in another order compares equal
/// to the same set read back out of `<DaysOfWeek>`. `5,1` and `1,5` are one schedule.
fn sorted_days(list: &str) -> String {
    let mut days: Vec<&str> = DAY_NAMES
        .iter()
        .copied()
        .filter(|d| list.split(',').any(|part| part.trim() == *d))
        .collect();
    days.dedup();
    days.join(",")
}

/// When a task fires, in one spelling both sides of the comparison can produce.
///
/// Task Scheduler keeps no file to diff, so the declaration is canonicalised from the `/SC`
/// arguments Shall would pass and the machine's copy is canonicalised from the trigger XML it
/// hands back. Anything neither can express is not canonicalised into something close — the
/// reader returns `None` and the reading is *unreadable*, which is the V.188 answer: a shape
/// Shall does not understand must never be reported as drift, or it re-writes the task on every
/// sync for ever.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn schtasks_when(args: &[String]) -> Option<String> {
    let sc = arg_after(args, "/SC")?;
    let every = arg_after(args, "/MO").unwrap_or("1");
    let at = arg_after(args, "/ST").unwrap_or("00:00");
    Some(match sc {
        "ONSTART" => "boot".to_string(),
        "MINUTE" => format!("every {}m", every),
        "HOURLY" => format!("every {}h from {}", every, at),
        "DAILY" => format!("daily at {}", at),
        "WEEKLY" => format!(
            "weekly on {} at {}",
            sorted_days(arg_after(args, "/D")?),
            at
        ),
        "MONTHLY" => {
            let day = arg_after(args, "/D").unwrap_or("1");
            match arg_after(args, "/M") {
                Some(month) => format!("monthly in {} on {} at {}", month, day, at),
                None => format!("monthly on {} at {}", day, at),
            }
        }
        _ => return None,
    })
}

/// The text between `<tag>` and `</tag>`, first occurrence, un-escaped for the five XML
/// entities Task Scheduler emits. Deliberately not a parser: the document is machine-written,
/// the elements wanted are leaves, and a dependency for four `find` calls is a dependency.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn xml_text<'a>(doc: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = doc.find(&open)? + open.len();
    let end = doc[start..].find(&close)? + start;
    Some(doc[start..end].trim())
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

/// How many trigger elements a `<Triggers>` block opens, whatever kind they are.
///
/// Any element whose name ends in `Trigger` counts — `<Triggers>` itself does not, and neither
/// does a closing tag. Shall writes exactly one.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn trigger_count(triggers: &str) -> usize {
    triggers
        .match_indices('<')
        .filter(|(at, _)| {
            let rest = &triggers[at + 1..];
            !rest.starts_with('/')
                && rest
                    .split(['>', ' ', '/'])
                    .next()
                    .is_some_and(|name| name.ends_with("Trigger"))
        })
        .count()
}

fn unescape_xml(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// `PT15M` / `PT6H` as the interval Shall spells `every 15m`.
///
/// Day-scale repetitions (`P1D`) deliberately have no reading: Shall writes a daily schedule as
/// a `ScheduleByDay` calendar trigger, never as a repetition, so a `P1D` under Shall's task name
/// is somebody else's task and describing it would be a guess.
fn iso_interval(spec: &str) -> Option<String> {
    let body = spec.strip_prefix('P')?;
    let time = body.strip_prefix('T')?;
    if let Some(m) = time.strip_suffix('M') {
        m.parse::<u32>().ok()?;
        return Some(format!("every {}m", m));
    }
    let h = time.strip_suffix('H')?;
    h.parse::<u32>().ok()?;
    Some(format!("every {}h", h))
}

/// `2010-12-16T03:00:00` as `03:00`.
fn boundary_time(boundary: &str) -> Option<String> {
    let time = boundary.split('T').nth(1)?;
    let (h, rest) = time.split_once(':')?;
    let m = rest.split(':').next()?;
    schtasks_time(h, m)
}

/// The canonical `when` for the single trigger in a task's XML, or `None` for any shape Shall
/// does not write and therefore cannot compare.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn when_from_xml(doc: &str) -> Option<String> {
    let triggers = xml_element(doc, "Triggers")?;
    // Shall writes exactly one trigger. More than one means somebody else has been here, and
    // describing what they did is not something this reader can do honestly.
    //
    // **Counted by shape, not against a list of the kinds this reader understands.** A list
    // would count the calendar trigger, miss the `<EventTrigger>` beside it, and report the
    // task as matching a declaration that says nothing about the event — which is the reader
    // claiming to have read something it never saw.
    if trigger_count(triggers) != 1 {
        return None;
    }

    if triggers.contains("<BootTrigger") {
        return Some("boot".to_string());
    }

    let at = xml_text(triggers, "StartBoundary").and_then(boundary_time);

    // A repetition is the only place Task Scheduler can put a sub-daily interval, whichever
    // trigger element carries it.
    if let Some(repetition) = xml_element(triggers, "Repetition") {
        let interval = iso_interval(xml_text(repetition, "Interval")?)?;
        return Some(match interval.strip_suffix(|c: char| c == 'h') {
            Some(_) => format!("{} from {}", interval, at?),
            None => interval,
        });
    }

    if let Some(daily) = xml_element(triggers, "ScheduleByDay") {
        // Shall only ever writes a one-day interval; anything else is not its task.
        if xml_text(daily, "DaysInterval").unwrap_or("1") != "1" {
            return None;
        }
        return Some(format!("daily at {}", at?));
    }

    if let Some(weekly) = xml_element(triggers, "ScheduleByWeek") {
        if xml_text(weekly, "WeeksInterval").unwrap_or("1") != "1" {
            return None;
        }
        let days = xml_element(weekly, "DaysOfWeek")?;
        let named: Vec<&str> = DAY_ELEMENTS
            .iter()
            .enumerate()
            .filter(|(_, element)| {
                days.contains(&format!("<{} ", element))
                    || days.contains(&format!("<{}/", element))
                    || days.contains(&format!("<{}>", element))
            })
            .map(|(i, _)| DAY_NAMES[i])
            .collect();
        if named.is_empty() {
            return None;
        }
        return Some(format!("weekly on {} at {}", named.join(","), at?));
    }

    if let Some(monthly) = xml_element(triggers, "ScheduleByMonth") {
        let day = xml_text(xml_element(monthly, "DaysOfMonth")?, "Day")?.to_string();
        let month = xml_element(monthly, "Months").and_then(|months| {
            MONTH_ELEMENTS
                .iter()
                .enumerate()
                .find(|(_, element)| {
                    months.contains(&format!("<{} ", element))
                        || months.contains(&format!("<{}/", element))
                        || months.contains(&format!("<{}>", element))
                })
                .map(|(i, _)| MONTH_NAMES[i])
        });
        return Some(match month {
            Some(m) => format!("monthly in {} on {} at {}", m, day, at?),
            None => format!("monthly on {} at {}", day, at?),
        });
    }

    None
}

/// A whole `schtasks /Query /XML` document as the canonical record it describes, or `None` for a
/// document this reader cannot honestly claim to have read.
///
/// Pure, and separate from the query, so it can be asked about a **verbatim** document captured
/// from a real Task Scheduler rather than only about documents this file made up.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn provisioned_from_xml(doc: &str, expected: Option<&str>) -> Option<Provisioned> {
    let exec = xml_element(doc, "Exec")?;
    let program = unescape_xml(xml_text(exec, "Command")?);
    let arguments = xml_text(exec, "Arguments")
        .map(unescape_xml)
        .unwrap_or_default();
    let command = format!("{} {}", program.trim().trim_matches('"'), arguments.trim());

    let when = when_from_xml(doc)?;
    let elevated = xml_element(doc, "Principals")
        .and_then(|p| xml_text(p, "RunLevel"))
        .is_some_and(|level| level == "HighestAvailable");
    // The task's own switch, which lives in `<Settings>`. A trigger carries an `<Enabled>` of its
    // own, so a document-wide search for the first one answers a different question. Absent
    // means armed — Task Scheduler omits the element on a task nobody has disabled, which is the
    // ordinary case and not a missing answer.
    let armed = xml_element(doc, "Settings")
        .and_then(|s| xml_text(s, "Enabled"))
        .map(|v| v == "true")
        .unwrap_or(true);
    let found = xml_element(doc, "Principals")
        .and_then(|p| xml_text(p, "UserId"))
        .filter(|user| !user.is_empty());
    let principal = expected.map(|want| match found {
        Some(found) if found == want => want.to_string(),
        other => other.unwrap_or_default().to_string(),
    });

    Some(Provisioned {
        spec: schtasks_spec(&command, &when, elevated, principal.as_deref()),
        armed,
    })
}

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn schtasks_spec(command: &str, when: &str, elevated: bool, account: Option<&str>) -> String {
    let mut out = format!(
        "run: {}\nwhen: {}\nlevel: {}\n",
        command.trim(),
        when,
        if elevated {
            "HighestAvailable"
        } else {
            "LeastPrivilege"
        }
    );
    if let Some(user) = account {
        out.push_str(&format!("account: {user}\n"));
    }
    out
}

/// Task Scheduler's `/XML` output, whichever width it came back in.
///
/// The document declares `encoding="UTF-16"` and arrives here as bytes; on this machine a piped
/// read gives one byte per character, and a redirect to a file gives two. Deciding from the
/// bytes rather than from the header covers both without a table of which Windows does which.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn decode_xml(bytes: &[u8]) -> String {
    let utf16 =
        bytes.starts_with(&[0xFF, 0xFE]) || (bytes.len() >= 4 && bytes[1] == 0 && bytes[3] == 0);
    if !utf16 {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    let body = bytes.strip_prefix(&[0xFF, 0xFE][..]).unwrap_or(bytes);
    // `as_chunks` rather than `chunks_exact(2)`: the pair arrives as `[u8; 2]`, so the two
    // indexes that could panic stop existing. A trailing odd byte is a truncated code unit and
    // is dropped either way.
    let (pairs, _odd_trailing_byte) = body.as_chunks::<2>();
    let units: Vec<u16> = pairs.iter().copied().map(u16::from_le_bytes).collect();
    String::from_utf16_lossy(&units)
}

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub struct WindowsTaskProvisioner;

impl WindowsTaskProvisioner {
    fn task_name(name: &str) -> String {
        format!("Shall_{}", name)
    }

    fn principal(target: &TaskTarget) -> Option<String> {
        let account = target.account.as_ref()?;
        Some(account.sid().unwrap_or_else(|| account.name().to_string()))
    }

    fn command_line(config: &ScheduleConfig, shall_bin: &Path) -> String {
        format!("{} {}", shall_bin.display(), config.command)
    }
}

#[async_trait]
impl TaskProvisioner for WindowsTaskProvisioner {
    fn refuse_unsupported(&self, config: &ScheduleConfig) -> std::result::Result<(), String> {
        // Both live in the task's XML and `schtasks` has no flag for either, so accepting them
        // would mean accepting an option and writing a task without it.
        if config.jitter.is_some() {
            return Err(
                "`jitter` is a Task Scheduler XML setting (`RandomDelay`) and `schtasks` \
                        has no flag for it, so Shall cannot put it on the task it creates. \
                        Randomise inside the command the schedule invokes, or drop the option."
                    .into(),
            );
        }
        if config.persistent.is_some() {
            return Err(
                "`persistent` is a Task Scheduler XML setting (`StartWhenAvailable`) and \
                        `schtasks` has no flag for it, so Shall can neither set it nor promise \
                        what it defaults to. Drop the option on this machine."
                    .into(),
            );
        }
        Ok(())
    }

    fn rendered(
        &self,
        config: &ScheduleConfig,
        shall_bin: &Path,
        target: &TaskTarget,
    ) -> std::result::Result<Provisioned, String> {
        let args = map_cron_to_schtasks(&config.cron)?;
        let when = schtasks_when(&args).ok_or_else(|| {
            format!(
                "`{}` maps to a Task Scheduler trigger Shall cannot describe.",
                config.cron
            )
        })?;
        Ok(Provisioned {
            spec: schtasks_spec(
                &Self::command_line(config, shall_bin),
                &when,
                config.elevated == Some(true),
                Self::principal(target).as_deref(),
            ),
            armed: armed(config),
        })
    }

    async fn read_task(
        &self,
        executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> Reading {
        let tn = Self::task_name(name);
        let queried = executor
            .run("schtasks", &["/Query", "/TN", &tn, "/XML"], false)
            .await;
        let doc = match queried {
            Ok(out) => decode_xml(&out.stdout),
            Err(_) => {
                // Two reasons a query fails and only one of them is an answer. Asking
                // `schtasks` a question it can always answer separates them, which the error
                // text cannot: `ERROR: The system cannot find the file specified.` is
                // translated on a non-English Windows and matching it would make the reading
                // depend on the machine's display language.
                return match executor
                    .run("schtasks", &["/Query", "/FO", "CSV", "/NH"], false)
                    .await
                {
                    Ok(_) => Reading::Absent,
                    Err(_) => Reading::Unreadable,
                };
            }
        };

        match provisioned_from_xml(&doc, Self::principal(target).as_deref()) {
            Some(held) => Reading::Holds(held),
            None => Reading::Unreadable,
        }
    }

    async fn add_task(
        &self,
        executor: &CommandExecutor,
        config: &ScheduleConfig,
        shall_bin: &Path,
        target: &TaskTarget,
    ) -> Result<()> {
        let name = Self::task_name(&config.name);
        // Quoted for `/TR`, which takes one string and splits it itself; the read-back compares
        // the unquoted form Task Scheduler stores.
        let cmd = format!("\"{}\" {}", shall_bin.display(), config.command);

        // Refused here, before anything is created: a schedule Task Scheduler cannot express
        // must not become the nearest one it can. `Refused` and not `Io` — Shall looked and
        // declined on purpose, which is exit code 3 (U21), and a script that retries on failure
        // must not retry this.
        let schedule = map_cron_to_schtasks(&config.cron)
            .map_err(|e| Error::Refused(format!("`schedule:{}`: {}", config.name, e)))?;

        if target.is_scoped() && !TaskTarget::can_register_for_another_account() {
            return Err(target.refuse_no_user_domain(
                &format!("schedule:{}", config.name),
                "Windows Task Scheduler",
                "registering a task for another account needs that account's password, and \
                 Shall does not hold it",
            ));
        }
        let principal = Self::principal(target);
        let mut args: Vec<&str> = vec!["/Create", "/TN", &name, "/TR", &cmd, "/F"];
        args.extend(schedule.iter().map(String::as_str));
        if config.elevated == Some(true) {
            args.extend(["/RL", "HIGHEST"]);
        }
        if let Some(user) = principal.as_deref() {
            args.extend(["/RU", user]);
        }

        // `ERROR: Access is denied.` is what Task Scheduler says when the shell is not
        // elevated, and on its own it names neither the cause nor the cure — it reads like a
        // permissions problem with the config. Registering a task needs an administrator here
        // whatever `/RU` says (measured), so say that instead of forwarding four words.
        executor.run("schtasks", &args, true).await.map_err(|e| {
            if e.to_string().to_lowercase().contains("access is denied") {
                Error::Permission(format!(
                    "creating the scheduled task `{}` needs an elevated shell — Windows Task \
                     Scheduler refuses to register one otherwise. Re-run `shall sync` from a \
                     terminal opened with \"Run as administrator\".",
                    name
                ))
            } else {
                e
            }
        })?;

        // `/Create` always produces an armed task, so the declaration that says otherwise is
        // applied after it rather than instead of it.
        if !armed(config) {
            executor
                .run("schtasks", &["/Change", "/TN", &name, "/DISABLE"], true)
                .await?;
        }
        Ok(())
    }

    async fn remove_task(
        &self,
        executor: &CommandExecutor,
        name: &str,
        target: &TaskTarget,
    ) -> Result<()> {
        let tn = Self::task_name(name);
        // `/Delete` on a task that does not exist exits non-zero, so the exit code cannot
        // tell "already gone" from "refused"; the end state can.
        let _ = executor
            .run("schtasks", &["/Delete", "/TN", &tn, "/F"], true)
            .await;
        if self.is_task_active(executor, name, target).await {
            return Err(Error::Io(format!(
                "the scheduled task `{}` still exists after removal. Check \
                 `schtasks /Query /TN {}`.",
                name, tn
            )));
        }
        Ok(())
    }

    async fn is_task_active(
        &self,
        executor: &CommandExecutor,
        name: &str,
        _target: &TaskTarget,
    ) -> bool {
        let tn = Self::task_name(name);
        match executor
            .run("schtasks", &["/Query", "/TN", &tn], false)
            .await
        {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }
}
