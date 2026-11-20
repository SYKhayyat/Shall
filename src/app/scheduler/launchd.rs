//! The scheduling dialect of launchd: agents, plists, and what `StartCalendarInterval` can say.

use super::{
    armed, remove_generated, unescape_xml, xml_element, xml_escape, HeldTask, Provisioned, Reading,
    TaskProvisioner, TaskTarget,
};
use crate::config::declaration::ScheduleConfig;
use crate::core::executor::StdOutput;
use crate::core::{CommandExecutor, Error, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a test
/// crate links the library without `cfg(test)`, so a private item is invisible there.
pub struct MacLaunchdProvisioner;
/// The same question for `LaunchAgents`, where the label is `com.shall.<name>.plist`.
/// **`pub` for the tests' sake**, and `None` means the same thing it means one function up.
pub fn agents_in(dir: &Path) -> Option<Vec<String>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        // No `LaunchAgents` directory is a machine that holds no agents; one that cannot be read
        // might hold anything, and is not the same answer.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Some(Vec::new()),
        Err(_) => return None,
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let file = entry.file_name();
        let Some(file) = file.to_str() else { continue };
        let Some(stem) = file.strip_suffix(".plist") else {
            continue;
        };
        let Some(name) = stem.strip_prefix("com.shall.") else {
            continue;
        };
        names.push(name.to_string());
    }
    names.sort();
    Some(names)
}

/// [`agents_in`] with each agent's command line, for the sweep that must not remove another
/// config's tasks (#182).
///
/// The argv is the plist's `ProgramArguments` array as written — one element per word of the
/// command, because launchd hands the array to `execve` whole (#195) — so the read-back is the
/// array itself, unescaped, not a re-split of a line. A plist whose array cannot be read
/// answers with an empty argv; the file's `com.shall.` label is the store's certification
/// that *Shall* wrote the agent.
pub fn agents_held(dir: &Path) -> Option<Vec<HeldTask>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Some(Vec::new()),
        Err(_) => return None,
    };
    let mut tasks: Vec<HeldTask> = Vec::new();
    for entry in entries.flatten() {
        let file = entry.file_name();
        let Some(file) = file.to_str() else { continue };
        let Some(stem) = file.strip_suffix(".plist") else {
            continue;
        };
        // Only a label that honours our prefix is one of ours at all — the `#158` gate, and
        // the certification that survives an unreadable `ProgramArguments` array.
        let Some(name) = stem.strip_prefix("com.shall.") else {
            continue;
        };
        tasks.push(HeldTask {
            name: name.to_string(),
            argv: std::fs::read_to_string(dir.join(file))
                .ok()
                .and_then(|doc| plist_program_arguments(&doc))
                .unwrap_or_default(),
            certified: true,
        });
    }
    tasks.sort_by(|a, b| a.name.cmp(&b.name));
    Some(tasks)
}

/// The `ProgramArguments` array of a plist *as written by Shall's own renderer*, as argv.
///
/// Scoped to the `ProgramArguments` key because the plist carries other arrays (`StartCalendar`
/// is built as an array too); the array's `<string>` children are unescaped with the same
/// `unescape_xml` the Windows read-back uses. An empty array is a plist with nothing to run —
/// a hand-edited agent — and answers `Some(vec![])`, the vocabulary the renderer itself uses.
pub fn plist_program_arguments(doc: &str) -> Option<Vec<String>> {
    let key = doc.find("<key>ProgramArguments</key>")?;
    let array = xml_element(&doc[key..], "array")?;
    let mut out = Vec::new();
    let mut rest = array;
    while let Some(at) = rest.find("<string>") {
        let value = &rest[at + "<string>".len()..];
        let end = value.find("</string>")?;
        out.push(unescape_xml(&value[..end]));
        rest = &value[end + "</string>".len()..];
    }
    Some(out)
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
            <string>{bin}</string>{roots}{cmd}\n</array>\n\
            {schedule}\n\
            {disabled}\
            <key>StandardOutPath</key><string>{log}</string>\n\
            <key>StandardErrorPath</key><string>{log}</string>\n\
            </dict>\n</plist>",
            label = label,
            bin = shall_bin.display(),
            roots = target
                .roots
                .as_argv()
                .iter()
                .map(|a| {
                    format!(
                        "<string>{}</string>",
                        xml_escape(a)
                    )
                })
                .collect::<String>(),
            // One `<string>` per word, because `ProgramArguments` goes to `execve` whole: a
            // single element holding `sync --dry-run` arrives as one argument and the schedule
            // dies in argument parsing on every firing (#195). The split is
            // `schedule::words`, the same one `refuse_self_named` asks for the first word.
            cmd = crate::model::schedule::words(&config.command)
                .iter()
                .map(|w| format!("<string>{}</string>", xml_escape(w)))
                .collect::<String>(),
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

    /// A directory scan of `LaunchAgents`, for the same reason the systemd one is: the plist is
    /// the unit, and `launchctl list` also names agents that are not files here.
    async fn installed(
        &self,
        _executor: &CommandExecutor,
        target: &TaskTarget,
    ) -> Option<Vec<String>> {
        // The directory is `plist_path`'s, without a name — one path decides where plists live.
        let probe = Self::plist_path("probe", target)?;
        let dir = probe.parent()?.to_path_buf();
        crate::core::blocking::off_the_runtime(move || agents_in(&dir))
            .await
            .ok()
            .flatten()
    }

    async fn installed_with_command(
        &self,
        _executor: &CommandExecutor,
        target: &TaskTarget,
    ) -> Option<Vec<HeldTask>> {
        let probe = Self::plist_path("probe", target)?;
        let dir = probe.parent()?.to_path_buf();
        crate::core::blocking::off_the_runtime(move || agents_held(&dir))
            .await
            .ok()
            .flatten()
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
