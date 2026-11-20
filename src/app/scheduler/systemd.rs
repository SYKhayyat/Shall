//! The scheduling dialect of systemd: user units, and the cron→OnCalendar map.

use super::{
    armed, remove_generated, HeldTask, Provisioned, Reading, TaskProvisioner, TaskTarget,
    UNIT_SEPARATOR,
};
use crate::config::config::ScheduleConfig;
use crate::core::executor::StdOutput;
use crate::core::{CommandExecutor, Error, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

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
             [Service]\nType=oneshot\nExecStart={bin}{roots_flags} {cmd}\n\
             StandardOutput=append:{log}\nStandardError=append:{log}\n",
            kind = if config.cron == "@reboot" {
                "Reboot Job"
            } else {
                "Job"
            },
            name = config.name,
            bin = shall_bin.display(),
            roots_flags = target.roots.flag_spellings(),
            // One `<string>` per word, because `ProgramArguments` goes to `execve` whole: a
            // single element holding `sync --dry-run` arrives as one argument and the schedule
            // dies in argument parsing on every firing (#195). The split is
            // `schedule::words`, the same one `refuse_self_named` asks for the first word.
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

    /// A directory scan, because systemd's user units *are* the directory — `systemctl list`
    /// would answer about the units it has loaded and not the files on disk, and the file is
    /// what survives (and what `read_task` reads).
    async fn installed(
        &self,
        _executor: &CommandExecutor,
        target: &TaskTarget,
    ) -> Option<Vec<String>> {
        let dir = Self::unit_dir(target)?;
        // A directory walk is a blocking call on a runtime worker; `off_the_runtime` is the
        // door this tree puts those behind (II.52).
        crate::core::blocking::off_the_runtime(move || units_in(&dir))
            .await
            .ok()
            .flatten()
    }

    async fn installed_with_command(
        &self,
        _executor: &CommandExecutor,
        target: &TaskTarget,
    ) -> Option<Vec<HeldTask>> {
        let dir = Self::unit_dir(target)?;
        crate::core::blocking::off_the_runtime(move || units_held(&dir))
            .await
            .ok()
            .flatten()
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
/// The schedules a systemd unit directory holds, by declared name — the filter behind
/// [`TaskProvisioner::installed`], written out so the rule can be read and tested without a
/// scheduler: **only a `shall-` name is a candidate**, and both halves of one job are one name.
///
/// **`None` is not `Some(vec![])`.** A directory that does not exist holds nothing; one that
/// cannot be read might hold anything, and reporting it as empty is how a converge walks away
/// from units it simply could not see.
///
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn units_in(dir: &Path) -> Option<Vec<String>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Some(Vec::new()),
        Err(_) => return None,
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let file = entry.file_name();
        let Some(file) = file.to_str() else { continue };
        // Both halves of one job carry the stem; the `@reboot` shape has a service only.
        let Some(stem) = file
            .strip_suffix(".service")
            .or_else(|| file.strip_suffix(".timer"))
        else {
            continue;
        };
        let Some(name) = stem.strip_prefix("shall-") else {
            continue;
        };
        if !names.iter().any(|n| n == name) {
            names.push(name.to_string());
        }
    }
    names.sort();
    Some(names)
}

/// [`units_in`] with each name's command line, for the sweep that must not remove another
/// config's tasks (#182).
///
/// The argv is the `ExecStart=` line of the `.service` half, split with the same
/// `schedule::words` the writer rendered it with. The `.timer` half carries no command (it is
/// the schedule, not the run); a name whose service file has no `ExecStart=` answers with an
/// empty argv — the file itself is the store's certification that *Shall* wrote the unit, and
/// `schedule_origin` decides the rest.
///
/// Same shape as [`units_in`]: `None` for a directory that cannot be read, `Some` otherwise;
/// names sorted, each name once however many files carry its stem.
pub fn units_held(dir: &Path) -> Option<Vec<HeldTask>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Some(Vec::new()),
        Err(_) => return None,
    };
    let mut tasks: Vec<HeldTask> = Vec::new();
    for entry in entries.flatten() {
        let file = entry.file_name();
        let Some(file) = file.to_str() else { continue };
        let Some(stem) = file
            .strip_suffix(".service")
            .or_else(|| file.strip_suffix(".timer"))
        else {
            continue;
        };
        let Some(name) = stem.strip_prefix("shall-") else {
            continue;
        };
        if let Some(existing) = tasks.iter_mut().find(|t| t.name == name) {
            // Both halves of one job are one name; only the service half carries a command,
            // and whichever half the directory answers first, readdir order must not decide
            // whether the roots are read (a task's argv *is* the #182 judgement).
            if file.ends_with(".service") && existing.argv.is_empty() {
                existing.argv = exec_start_of(&dir.join(file));
            }
            continue;
        }
        let argv = if file.ends_with(".service") {
            exec_start_of(&dir.join(file))
        } else {
            Vec::new()
        };
        tasks.push(HeldTask {
            name: name.to_string(),
            argv,
            // The unit file honours Shall's prefix in the directory Shall's own renderer
            // writes into — that is the `#158` gate, and it is answerable even when the
            // `ExecStart=` line is not.
            certified: true,
        });
    }
    tasks.sort_by(|a, b| a.name.cmp(&b.name));
    Some(tasks)
}

/// The argument vector of a `.service` unit's `ExecStart=` line, if the file has one.
///
/// Only the plain `ExecStart=` spelling Shall writes is read; a hand-edited unit's `-`/`@`
/// prefixes are treated as no command at all, and the store's certification then says what it
/// is. A read failure answers empty rather than poisoning the listing — `units_in`'s rule for
/// a single name.
fn exec_start_of(service: &Path) -> Vec<String> {
    let contents = match std::fs::read_to_string(service) {
        Ok(contents) => contents,
        Err(_) => return Vec::new(),
    };
    for line in contents.lines() {
        if let Some(rest) = line.trim().strip_prefix("ExecStart=") {
            return crate::model::schedule::words(rest);
        }
    }
    Vec::new()
}
