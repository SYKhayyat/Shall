use crate::app::scheduler::Standing;
use crate::core::Result;
use std::collections::BTreeSet;
use tracing::info;

/// Schedules holds only what it uses. It is built from an [`App`](crate::app::App) by
/// `App::schedules()` and can be built without one.
pub struct Schedules<'a> {
    pub(crate) config: &'a std::sync::Arc<crate::config::Config>,
    pub(crate) executor: &'a crate::core::CommandExecutor,
    pub(crate) scheduler: &'a std::sync::Arc<crate::app::scheduler::SchedulerManager>,
    /// The guard asks a backend's own registry for the OS-essential set, and a schedule is not a
    /// package — but the token it mints is the one every removal path takes, and a converge that
    /// took units off the machine without asking is exactly what `guard::Reaped` exists to stop.
    pub(crate) registry: &'a std::sync::Arc<crate::verbs::prelude::BackendRegistry>,
}

impl Schedules<'_> {
    /// Provision the declared `schedule:` lines onto the OS scheduler (S21) — II.7's schedule
    /// phase, after packages and dependents. Each line is mapped to a `ScheduleConfig` (which
    /// validates it carries `cron` and `run`) and handed to the `SchedulerManager`. Declarative
    /// and idempotent: re-registering the same task each sync is how the system state is kept
    /// equal to what the `schedules` file says.
    pub async fn apply(&self, state: &crate::model::DesiredState) -> Result<()> {
        for (name, opts, origin) in state.schedules() {
            let cfg = crate::model::schedule::schedule_config(
                name,
                opts,
                origin,
                &self.config.guard.never_unattended,
            )?;
            // Asked before the preview and before the work, so `plan` and `sync` describe the
            // same machine. A schedule that already says what the line says is left alone: it
            // was always re-registered on every run, which converged and reported nothing, and
            // reporting nothing about a schedule being rewritten is the defect this closes.
            let because = match self.scheduler.standing(self.executor, &cfg).await {
                Standing::Matches => continue,
                Standing::Missing => "it is not registered",
                Standing::Differs(what) => what,
                // **Unreadable is not overwritable.** Provisioning is `/Create /F`, which
                // clobbers whatever sits in the way — and "whatever" includes a task a hand
                // edited, on the word of a scheduler that could not even say what it holds.
                // This arm used to fall through and provision anyway, every sync, while
                // `scheduler`'s own doctrine for the same situation says the opposite.
                Standing::Unknown => {
                    return Err(crate::core::Error::Refused(format!(
                        "schedule `{}` cannot be read back from this scheduler, so Shall \
                         will not overwrite it: provisioning would clobber a registration \
                         nobody has read. Remove or fix the existing task first.",
                        name
                    )));
                }
            };
            if self.config.dry_run {
                crate::would!(
                    "would schedule `{}`: `{}` on `{}` — {}",
                    name,
                    cfg.command,
                    cfg.cron,
                    because
                );
                continue;
            }
            info!(
                "Schedule: provisioning `{}` ({}) — `{}` on `{}`, because {}",
                name, origin, cfg.command, cfg.cron, because
            );
            self.scheduler.provision(self.executor, &cfg).await?;
        }
        self.remove_what_is_no_longer_declared(state).await
    }

    /// Take off this machine a schedule the files no longer declare (`#158`).
    ///
    /// Every other arm of this phase starts from a line and asks the machine about it; a unit
    /// whose line is gone is invisible to all of them, which is how the owner's
    /// `shall-nightly.timer` kept firing at 03:00 for a week after its `schedules` file had been
    /// deleted — the machine still ran a job the files did not describe, and nothing said so.
    ///
    /// **Only what Shall wrote, and only what this run wrote (`#182`).** The provisioner lists
    /// by its own prefix (`shall-`, `com.shall.`, `Shall_`), so a task a person registered by
    /// hand is not a candidate; a scheduler that could not be asked contributes nothing rather
    /// than emptying the machine (`TaskProvisioner::installed`); and the command line each task
    /// fires decides whose it is (`SchedulerManager::origin_of`). A task that names roots this
    /// run did not use was declared by another config, and a task that records no roots and
    /// would fire another binary was written before `c7dd0b67` by someone else — `sync` in one
    /// config used to delete both on a machine with two configs, which is the defect `had` to
    /// stop being this phase's job. They stay registered, and this run says so every time,
    /// because a sweep that silently skips is a sweep nobody can trust to have looked.
    ///
    /// **Through the guard, like every other removal.** The token is asked for with the keys
    /// being taken and the removal happens only if it authorises the whole set — a schedule that
    /// starts a service is not a smaller removal than a package, and the guard is what makes
    /// "Shall decided" a fact about the size of the ask.
    async fn remove_what_is_no_longer_declared(
        &self,
        state: &crate::model::DesiredState,
    ) -> Result<()> {
        let declared: BTreeSet<String> = state
            .schedules()
            .map(|(name, _, _)| name.to_string())
            .collect();
        let Some(held) = self.scheduler.installed_tasks(self.executor, None).await else {
            // Unreadable is not empty: claiming this machine holds no schedules because the
            // scheduler could not be asked is the false convergence this whole module refuses.
            return Ok(());
        };
        let mut ours: Vec<String> = Vec::new();
        for task in held {
            if declared.contains(task.name.as_str()) {
                continue;
            }
            let name = &task.name;
            match self.scheduler.origin_of(&task) {
                // The line's roots are this run's, the binary is this binary, or the store
                // certifies Shall wrote a unit with no readable line. The `#158` sweep's
                // business, exactly as before.
                crate::app::scheduler::ScheduleOrigin::Ours => ours.push(task.name),
                crate::app::scheduler::ScheduleOrigin::AnotherConfig { config, data } => {
                    let found = [
                        config.map(|v| format!("--config-dir {}", v)),
                        data.map(|v| format!("--data-dir {}", v)),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" ");
                    tracing::warn!(
                        "Schedule: `{}` is no longer declared and another config registered it \
                         (it fires with {}), so this sync left it in place (#182).",
                        name,
                        found
                    );
                }
                crate::app::scheduler::ScheduleOrigin::ForeignBinary { binary } => {
                    tracing::warn!(
                        "Schedule: `{}` is no longer declared and would fire `{}`, a different \
                         Shall binary, so this sync left it in place (#182).",
                        name,
                        binary
                    );
                }
                crate::app::scheduler::ScheduleOrigin::Unreadable => {
                    tracing::warn!(
                        "Schedule: `{}` is no longer declared but its command line could not be \
                         read back and nothing else certifies who registered it, so this sync \
                         left it in place (#182).",
                        name
                    );
                }
            }
        }
        if ours.is_empty() {
            return Ok(());
        }
        // `warn!` and `would_warn!`, the pair `extras.rs` uses for the same kind of change: what
        // the files stopped declaring is not routine progress, and it must be visible at the
        // default log level — the defect `#153` was a real change reported below that floor.
        if self.config.dry_run {
            crate::would_warn!(
                "would remove {} schedule(s) this machine still runs and the files no longer \
                 declare: {}",
                ours.len(),
                ours.join(", ")
            );
            return Ok(());
        }

        let keys: Vec<(String, String)> = ours
            .iter()
            .map(|name| ("schedule".to_string(), name.clone()))
            .collect();
        let reaped = crate::app::sync::guard::enforce_extras(
            self.config,
            self.registry,
            &keys,
            &crate::app::sync::guard::Reaping::new(),
            crate::app::sync::guard::GuardScope::Sync,
        )
        .await?;
        if !reaped.authorises(ours.len()) {
            return Err(crate::core::Error::Refused(format!(
                "{} schedule(s) are no longer declared and the guard did not clear their \
                 removal: {}",
                ours.len(),
                ours.join(", ")
            )));
        }
        let mut left_behind: Vec<String> = Vec::new();
        for name in &ours {
            // **One orphan that will not come off is not a failed sync.** This was `?`, and the
            // consequence was measured: a task this shell may not delete (Task Scheduler answers
            // `Access is denied.` to a filtered token) failed *every* `sync` on the machine, over
            // a schedule the run had not declared and could not remove — so a converge whose every
            // declared line was already in effect reported failure (#173).
            //
            // `extras::reconcile` already answers this exact question the other way and says so in
            // its own doc — *"a backend that cannot undo one extra must not block the rest, so
            // each failure warns and the run continues"* — and the two paths remove schedules by
            // the same means. This is that rule, applied to the path that asks the machine instead
            // of the ledger.
            //
            // **Not silent.** The name, the reason the OS gave and the fact that it is still
            // registered are all reported, and the next run tries again, so the state is visible
            // and it does not go stale.
            if let Err(e) = self
                .scheduler
                .deprovision(self.executor, name, None, reaped)
                .await
            {
                tracing::warn!(
                    "Schedule: `{}` is no longer declared and could not be removed, so it is \
                     still registered on this machine and still fires: {}",
                    name,
                    e
                );
                left_behind.push(name.clone());
            }
        }
        let removed: Vec<&str> = ours
            .iter()
            .filter(|n| !left_behind.contains(n))
            .map(String::as_str)
            .collect();
        if removed.is_empty() {
            return Ok(());
        }
        tracing::warn!(
            "Schedule: removed {} schedule(s) this machine still ran and the files no longer \
             declare: {}. Shall put them there and its records did not name them any more.",
            removed.len(),
            removed.join(", ")
        );
        Ok(())
    }
}
