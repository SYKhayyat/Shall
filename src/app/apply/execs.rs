use crate::core::LockFile;
use crate::core::{Error, Result};
use tracing::{debug, info, warn};

/// Execs holds only what it uses. It is built from an [`App`](crate::app::App) by
/// `App::execs()` and can be built without one.
pub struct Execs<'a> {
    pub(crate) config: &'a std::sync::Arc<crate::config::Config>,
    pub(crate) executor: &'a crate::core::CommandExecutor,
    pub(crate) registry: &'a std::sync::Arc<crate::backends::BackendRegistry>,
    pub(crate) journal: &'a std::sync::Arc<tokio::sync::Mutex<crate::core::Journal>>,
    pub(crate) reaping: &'a std::sync::Arc<crate::app::sync::guard::Reaping>,
}

/// What one `exec:` line resolves to: a script the config carries, or a step Shall ships.
///
/// Two arms rather than two code paths — everything after this point (the ceiling, the ledger,
/// the write-ahead record, the dry-run note) is the same for both, which is the argument for
/// `H8` being a catalogue feeding this statement instead of a second statement of its own.
enum Planned {
    Script(std::path::PathBuf),
    Step(crate::model::step::Step),
}

impl Execs<'_> {
    /// Resolve one `exec:` line to the script Shall would run, its content hash, and what the
    /// two ledgers say about it. Shared by the preview and the run so a plan cannot describe a
    /// decision the sync then makes differently.
    ///
    /// The path is taken relative to the config repo when it is not absolute — the script
    /// travels with the configuration that declares it, the way a `link:` source does.
    fn exec_plan(
        &self,
        script: &str,
        opts: &crate::config::grammar::Options,
        hooks: &crate::core::hook_lock::HookLedger,
        runs: &crate::core::ExecLedger,
    ) -> Result<(Planned, String, crate::model::exec::Decision)> {
        use crate::core::hook_lock::{exec_id, hash_script};

        // **A catalogued step has no file, no approval and no shebang (`H8`).** It is a row
        // compiled into this binary, so `II.12`'s question — *has a human read this code?* — was
        // answered by installing Shall. The ceiling still applies and still comes from the row
        // unless the line overrides it, so a step is counted by the same ledger as a script and
        // a release that changes a step's command counts it as new work.
        if let Some(name) = crate::model::step::named(script) {
            let step = crate::model::step::find(name).ok_or_else(|| {
                Error::Validation(format!(
                    "`exec:{}` names no step Shall ships on this machine. Available here: {}.",
                    script,
                    match crate::model::step::names_here().join(", ") {
                        empty if empty.is_empty() => "none".to_string(),
                        list => list,
                    }
                ))
            })?;
            let hash = crate::model::step::fingerprint(&step);
            let ceiling = crate::core::Ceiling::read(Some(opts.one("runs").unwrap_or(&step.runs)));
            let decision = crate::model::exec::Decision::of(
                &crate::core::hook_lock::Verdict::Approved,
                runs.count(&hash),
                ceiling,
            );
            return Ok((Planned::Step(step), hash, decision));
        }

        let declared = std::path::Path::new(script);
        let path = if declared.is_absolute() {
            declared.to_path_buf()
        } else {
            self.config.config_root().join(declared)
        };
        // The permission gate (R6) runs before the content is even read: a script whose mode
        // word fails the `[exec] trust` setting is refused on its permissions alone, in the
        // preview as well as the run, because both go through this one resolution.
        match permission_verdict(&path, self.config.exec.trust)? {
            PermVerdict::Accept => {}
            PermVerdict::Warn(why) => warn!("exec:{} — {}", script, why),
            PermVerdict::Refuse(why) => {
                return Err(Error::Refused(format!("exec:{} — {}", script, why)))
            }
        }
        let body = std::fs::read_to_string(&path).map_err(|e| {
            Error::Validation(format!(
                "`exec:{}` — cannot read the script at {} ({}). An `exec:` names a file the \
                 config carries; its contents are what Shall hashes and runs.",
                script,
                path.display(),
                e
            ))
        })?;
        let hash = hash_script(&body);
        let decision = crate::model::exec::Decision::of(
            &hooks.verdict(&exec_id(script), &hash),
            runs.count(&hash),
            crate::core::Ceiling::read(opts.one("runs")),
        );
        Ok((Planned::Script(path), hash, decision))
    }
    /// Print what each declared `exec:` will do, before anything happens (XIII.3's exit
    /// condition): the content hash, how many times that content has run here, and the
    /// decision that follows. Uses the same `exec_plan` the run uses, so the preview cannot
    /// describe one thing and the sync do another.
    ///
    /// A script that cannot be read is reported here rather than propagated: this is the
    /// preview, and the run raises the same problem as a real error a moment later.
    pub fn print_plan(&self, state: &crate::model::DesiredState, verb: crate::model::exec::Verb) {
        use crate::core::hook_lock::HookLedger;

        if !state.has_execs() {
            return;
        }
        let locks = self.config.layout().locks_dir();
        let (Ok(hooks), Ok(runs)) = (
            HookLedger::load(&HookLedger::path_in(&locks)),
            crate::core::ExecLedger::load(&crate::core::ExecLedger::path_in(&locks)),
        ) else {
            return;
        };
        println!("Scripts:");
        // **Every declared line, including the ones this verb will not run.** `execs_for` is
        // the running list and it is the wrong list to preview from: an `@on=upgrade` step
        // filtered out here is declared code that no preview anywhere shows, which is the
        // reporting hole `F12` was — a category dropped from the summary because the summary
        // was built from the actor's list rather than the reader's. The line says which verb
        // claims it instead, so nothing is hidden and nothing is misattributed.
        for (script, opts, origin) in state.execs() {
            let mine = verb.claims_line(script, opts.one("on"));
            match self.exec_plan(script, opts, &hooks, &runs) {
                Ok((_, hash, decision)) => {
                    println!("  exec:{}  ({})", script, origin);
                    match mine {
                        true => println!("    {}", decision.describe(&hash)),
                        // Not "will run — not this command", which reads as a contradiction
                        // and was what the first version printed. The decision is stated once,
                        // about the verb that owns the line.
                        false => println!(
                            "    {}, under `shall {}`",
                            decision.describe(&hash),
                            self.on_of(script, opts)
                        ),
                    }
                }
                Err(e) => println!("  exec:{}  ({}) — {}", script, origin, e),
            }
        }
    }
    /// Run the declared `exec:` scripts (XIII.3) — II.7's verb phase, after the packages and
    /// dependents a script is likely to depend on.
    ///
    /// Three things this does that a naive "run the command" would not: it refuses a script
    /// II.12 has not approved (a repo that can run code is the hook question with a different
    /// file name, and `-y` cannot approve); it runs a given *content* only as many times as its
    /// `@runs=` ceiling allows, so a settled sync executes nothing; and it records the run only
    /// when the script actually succeeded — a failed script has not happened, so the next sync
    /// must try it again.
    pub async fn apply(
        &self,
        state: &crate::model::DesiredState,
        verb: crate::model::exec::Verb,
        // How many packages this run actually moved, or `None` when the run cannot know —
        // a native whole-system `apt upgrade` reports no per-package count. `None` is NOT
        // zero: an `@after=` step is run rather than skipped, because the path that cannot
        // count is the path that moves the most, and skipping a firmware step after a
        // whole-system upgrade is the wrong direction to be wrong in.
        moved: Option<usize>,
        // The count is the number of UNDOS this pass performed. It exists because a
        // converged sync reports `already up to date` from a branch that can still run one,
        // and a summary that says nothing happened over a script that ran is the disease
        // `G8` is about. Scripts that RUN are not counted here: `Reconciled::applied` is
        // built from the package plan on every other path, and adding them only on this one
        // would make the number mean two things.
    ) -> Result<usize> {
        use crate::core::hook_lock::HookLedger;

        // No early return when nothing is declared: deleting the LAST `exec:` line is a real
        // change, and a teardown that only runs when something is still declared can never
        // undo the last one (S20 taught this for extras; it is the same shape here).
        let locks = self.config.layout().locks_dir();
        let hooks = HookLedger::load(&HookLedger::path_in(&locks))?;
        let runs_path = crate::core::ExecLedger::path_in(&locks);
        let mut runs = crate::core::ExecLedger::load(&runs_path)?;

        for (script, opts, origin) in state.execs_for(verb) {
            let (planned, hash, decision) = self.exec_plan(script, opts, &hooks, &runs)?;
            // **A step for a tool this machine does not have is skipped, not failed.** The
            // catalogue is the same on every machine and machines are not: a config shared
            // between a laptop with `rustup` and a server without it declares the step once,
            // and the server has nothing to do rather than something to report. Said out loud
            // rather than silently, because a step that never runs and never says so is
            // indistinguishable from one that is quietly broken.
            // `@after=N` — the step asked to wait for a run that moved enough to be worth
            // its while. Said out loud: a step that silently does not run is indistinguishable
            // from one that is broken, which is the same argument the skip below makes.
            if let Some(threshold) = opts
                .one("after")
                .and_then(|v| v.trim().parse::<usize>().ok())
            {
                if let Some(count) = moved {
                    if count < threshold {
                        info!(
                            "skipping exec:{} — it runs after {} package(s) move and this run \
                             moved {} ({})",
                            script, threshold, count, origin
                        );
                        continue;
                    }
                }
            }
            if let Planned::Step(step) = &planned {
                if !self.executor.command_exists_sync(&step.detect) {
                    info!(
                        "skipping exec:{} — `{}` is not on this machine ({})",
                        script, step.detect, origin
                    );
                    continue;
                }
            }
            if let crate::model::exec::Decision::NeedsApproval(verdict) = &decision {
                // A refusal, not a warning: this is code from the configuration, and II.12's
                // whole point is that nothing runs it until a human has looked.
                return Err(Error::Validation(format!(
                    "{}: {}",
                    origin,
                    crate::core::hook_lock::refusal(
                        &crate::core::hook_lock::exec_id(script),
                        "exec script",
                        verdict
                    )
                )));
            }
            if !decision.will_run() {
                debug!("exec:{} — {}", script, decision.describe(&hash));
                continue;
            }
            if self.config.dry_run {
                crate::would!("would run exec:{} ({})", script, origin);
                continue;
            }
            // `@runs=always` is named in the line it produces: a script that runs every sync
            // makes the sync non-idempotent, and the next person debugging a slow sync needs a
            // thread to pull (U13). A counted or once script does not need the note.
            if opts.one("runs") == Some("always") {
                info!(
                    "running exec:{} (runs=always — every sync) ({})",
                    script, origin
                );
            } else {
                info!("running exec:{} ({})", script, origin);
            }
            // Written and flushed BEFORE the interpreter starts, which is the whole point of a
            // write-ahead record: an entry made afterwards describes a mutation that already
            // happened, and the case it exists for is the one where "afterwards" never comes.
            // `exec:` is the only thing a sync runs that recovery cannot finish — a package can
            // be installed again, a `service:` re-converged from its line, but a script that
            // got half way has no recorded progress and no declared end state. So this record
            // buys the one thing left: the next run says what was interrupted instead of
            // silently running it again from the top.
            let started = self
                .record_start(crate::core::journal::JournalAction::Exec {
                    script: script.to_string(),
                    hash: hash.clone(),
                })
                .await;
            let outcome = self.run_planned(&planned).await;
            self.resolve(started, &outcome).await;
            outcome?;
            // Recorded only on success. A script that failed did not happen, and the next sync
            // must be free to try it again.
            runs.record_run(
                &hash,
                chrono::Utc::now().to_rfc3339(),
                script,
                opts.one("undo"),
            );
            runs.save(&runs_path)?;
        }

        self.undo_departed_execs(state, &mut runs, &runs_path).await
    }
    /// Run the `@undo=` of every `exec:` whose line has gone away, then forget it (U3).
    ///
    /// **The undo is read from the ledger, not from the files**, because by the time it is
    /// needed the declaration has been deleted — that is what removal means. Reading the
    /// current config would find nothing and do nothing, which is the `link:` source-deletion
    /// mistake wearing a different hat.
    ///
    /// A script that declared no `@undo=` is simply forgotten: Shall cannot invent an inverse,
    /// and pretending to would be worse than saying nothing. `plan` says so in those words.
    async fn undo_departed_execs(
        &self,
        state: &crate::model::DesiredState,
        runs: &mut crate::core::ExecLedger,
        runs_path: &std::path::Path,
    ) -> Result<usize> {
        // **Declared means reached by THIS resolution, from anywhere.** The file scan below
        // is how a *deleted line* is detected — but a `generate:` command can emit `exec:`
        // lines too, and those reach the desired state with no file behind them. Scanning
        // files alone made every generated exec look departed the moment its generator's
        // output changed shape, firing its undo while it was still declared.
        let mut declared = self.declared_exec_paths()?;
        for (script, _, _) in state.execs() {
            declared.insert(script.to_string());
        }
        // An unreadable configuration yields an empty set, which must never be read as "every
        // script departed" — that would run every undo on the machine because of a stray brace.
        if declared.is_empty() && state.has_execs() {
            return Ok(0);
        }
        let departed = runs.departed(&declared);
        if departed.is_empty() {
            return Ok(0);
        }

        // An `@undo=` is an arbitrary shell command a human wrote, and nothing can inspect it
        // for what it will remove — which is exactly why the total ceiling is the one gate it
        // answers. Charged as one set, before the first command runs, like every other
        // removal family; `--allow-mass-removal` is what answers a refusal here. Only rows
        // that WILL run are charged: a script that declared no undo is forgotten, not
        // executed, and forgetting is not a mutation.
        let runnable = departed
            .iter()
            .filter(|(_, record)| {
                record
                    .undo
                    .as_deref()
                    .map(|u| !u.trim().is_empty())
                    .unwrap_or(false)
            })
            .count();
        if !self.config.dry_run && runnable > 0 {
            crate::app::sync::guard::charge_unmodelled(
                self.config,
                self.reaping,
                runnable,
                crate::app::sync::guard::GuardScope::Apply,
            )?;
        }

        let mut undone = 0usize;
        for (hash, record) in departed {
            let name = record.script.as_deref().unwrap_or(&hash);
            let Some(undo) = record.undo.as_deref().filter(|u| !u.trim().is_empty()) else {
                debug!("`exec:{}` is no longer declared; it had no `undo`.", name);
                if !self.config.dry_run {
                    runs.forget(&hash);
                }
                continue;
            };
            if self.config.dry_run {
                crate::would!("would undo `exec:{}` with: {}", name, undo);
                continue;
            }
            info!("`exec:{}` is no longer declared — running its undo.", name);
            // An `@undo=` is an arbitrary shell command a human wrote, and it is the second of
            // the two mutations a sync makes that nothing can recompute. Same rule as the
            // script above: recorded before it starts, resolved after.
            let started = self
                .record_start(crate::core::journal::JournalAction::ExecUndo {
                    script: name.to_string(),
                    command: undo.to_string(),
                })
                .await;
            let outcome = self.run_shell_command(undo).await;
            self.resolve(started, &outcome).await;
            match outcome {
                Ok(()) => {
                    runs.forget(&hash);
                    undone += 1;
                }
                // Kept in the ledger on failure, so the next sync tries again rather than
                // forgetting an undo that never happened.
                Err(e) => warn!(
                    "could not undo `exec:{}` ({}); it stays recorded and the next sync will \
                     try again.",
                    name, e
                ),
            }
        }
        runs.save(runs_path)?;
        Ok(undone)
    }
    /// Every `exec:` script path the configuration contains — read from the FILES, ignoring
    /// `when` and ignoring which profiles are active.
    ///
    /// **This is deliberately not `state.execs()`.** That answers *does this machine want it
    /// right now*, which is a different question with a dangerous difference: a `when` that
    /// went false would read as a deleted line and run its `@undo=` — the enrol script
    /// un-enrolling itself on the sync after it succeeded, which is the flapping failure
    /// XIII.3 spends a section warning about. Deactivating a profile is likewise not a
    /// deletion. Only removing the line from the file is, and only the file can say so.
    ///
    /// A file that cannot be parsed contributes nothing rather than being treated as empty:
    /// concluding "every exec: has departed" from a syntax error would run every undo on the
    /// machine because of a stray brace.
    fn declared_exec_paths(&self) -> Result<std::collections::BTreeSet<String>> {
        use crate::config::grammar::Statement;

        let mut out = std::collections::BTreeSet::new();
        let modules = self.config.layout().modules_dir();
        let Ok(entries) = std::fs::read_dir(&modules) else {
            return Ok(out);
        };
        let known = |name: &str| self.registry.get(name).is_some();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("txt") {
                continue;
            }
            let Ok(body) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(doc) = crate::config::grammar::parse_document(&path, &body, &known) else {
                // Unparseable: the resolver reports it properly elsewhere. Here it must not be
                // read as "this file declares nothing".
                warn!(
                    "{}: could not be parsed while looking for `exec:` lines; leaving its \
                     scripts recorded.",
                    path.display()
                );
                return Ok(std::collections::BTreeSet::new());
            };
            for (stmt, _, _) in doc.every_statement() {
                if let Statement::Exec(script, _) = stmt {
                    out.insert(script.clone());
                }
            }
        }
        Ok(out)
    }
    /// Open a write-ahead entry for a mutation that is about to start.
    ///
    /// A journal that cannot be written must not stop the script running — the log exists to
    /// describe work, not to gate it, and refusing to converge a machine because a lock file
    /// is read-only would be the tail wagging the dog. It is said out loud, because a run
    /// nothing recorded is a run `heal` cannot account for and the user is owed that fact.
    async fn record_start(&self, action: crate::core::journal::JournalAction) -> Option<String> {
        let described = action.key();
        match self.journal.lock().await.record_start(action) {
            Ok(id) => Some(id),
            Err(e) => {
                warn!(
                    "could not record `{}` in the write-ahead log ({}); it will still run, but \
                     an interruption will not be reported by the next sync.",
                    described, e
                );
                None
            }
        }
    }

    /// Close the entry `record_start` opened. Paired with it here rather than at each call
    /// site, so an outcome cannot be recorded against the wrong id or forgotten entirely —
    /// forgotten is the one that matters, because an entry left open keeps `needs_recovery`
    /// true and re-reports the same script in front of every sync for ever.
    async fn resolve(&self, id: Option<String>, outcome: &Result<()>) {
        let Some(id) = id else {
            return;
        };
        let mut journal = self.journal.lock().await;
        let _ = match outcome {
            Ok(()) => journal.record_success(&id),
            Err(e) => journal.record_failure(&id, &e.to_string()),
        };
    }

    /// Run a command line through the platform's shell. Used only for `@undo=`, which is
    /// written as a command rather than a script path.
    async fn run_shell_command(&self, command: &str) -> Result<()> {
        #[cfg(windows)]
        let (program, args) = (
            "powershell",
            vec![
                "-NoProfile".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-Command".to_string(),
                command.to_string(),
            ],
        );
        #[cfg(not(windows))]
        let (program, args) = ("sh", vec!["-c".to_string(), command.to_string()]);
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.executor.run(program, &refs, false).await.map(|_| ())
    }
    /// Execute one script through the interpreter its first line names, or this platform's
    /// shell if it names none — `sh` on Unix and PowerShell on Windows. A repo that must ship
    /// two spellings of every script is a repo that cannot be shared, which is the reason the
    /// file travels with the config at all.
    /// Which verb a line belongs to, for the preview's sentence — the line's `@on=` if it has
    /// one, else the row's if it is a catalogued step, else `sync`. The same order
    /// [`crate::model::exec::Verb::claims_line`] decides by, so the preview cannot name one verb
    /// while the filter uses another.
    fn on_of(&self, script: &str, opts: &crate::config::grammar::Options) -> String {
        if let Some(explicit) = opts.one("on") {
            return explicit.to_string();
        }
        crate::model::step::named(script)
            .and_then(crate::model::step::find)
            .map(|s| s.on)
            .unwrap_or_else(|| "sync".to_string())
    }

    /// Run what this line resolved to.
    async fn run_planned(&self, planned: &Planned) -> Result<()> {
        match planned {
            Planned::Script(path) => self.run_exec_script(path).await,
            // Argv straight to the executor: a shipped row is data, and data that reaches a
            // shell stops being data. There is no shebang to read and no file to read it from.
            Planned::Step(step) => {
                let (program, args) = crate::model::step::launch(step).ok_or_else(|| {
                    Error::Validation(format!(
                        "the shipped step `{}` has an empty command",
                        step.name
                    ))
                })?;
                let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                self.executor.run(&program, &refs, false).await.map(|_| ())
            }
        }
    }

    async fn run_exec_script(&self, path: &std::path::Path) -> Result<()> {
        // A script that cannot be read is a refusal, not an empty body: `sh -c ""` exits 0,
        // and a script deleted between the plan and this line would otherwise be recorded as
        // a success against its old hash. The plan promised this file exists; if it does not,
        // say so instead of succeeding at nothing.
        let bytes = tokio::fs::read(path).await.map_err(|e| {
            Error::Validation(format!(
                "cannot read `exec:` script {}: {}",
                path.display(),
                e
            ))
        })?;
        // A script that is not UTF-8 has no first line this can read, and falls through to the
        // platform default — which is what every `exec:` script got before shebangs were read.
        // The text is used only to find that first line, so lossy replacement cannot corrupt
        // what runs: the interpreter is handed the path, not these bytes.
        let contents = String::from_utf8_lossy(&bytes);
        let launch = crate::model::script::launch_for(path, &contents)?;
        let refs: Vec<&str> = launch.args.iter().map(String::as_str).collect();
        self.executor
            .run(&launch.program, &refs, false)
            .await
            .map(|_| ())
    }
}

/// What the mode word of an `exec:` script means under the configured trust level.
#[derive(Debug)]
pub(crate) enum PermVerdict {
    Accept,
    /// Reported and survived, under `trust = "warn"`.
    Warn(String),
    /// The run stops here. `Error::Refused`, so exit 3 and no retry policy retries it.
    Refuse(String),
}

/// Pure over the POSIX mode bits so every platform's suite tests it; only reading the bits
/// out of a file is unix-only.
pub(crate) fn judge_script_perms(mode: u32, trust: crate::config::ExecTrust) -> PermVerdict {
    use crate::config::ExecTrust;
    let group_w = mode & 0o020 != 0;
    let world_w = mode & 0o002 != 0;
    let describe = |bits: &str| {
        format!(
            "the script is writable by {} (`{:o}`), which `trust = \"{}\"` does not allow",
            bits,
            mode & 0o777,
            match trust {
                ExecTrust::OwnerOnly => "owner-only",
                ExecTrust::NotWorldWritable => "not-world-writable",
                ExecTrust::Warn => "warn",
            }
        )
    };
    let warn = if group_w || world_w {
        Some(format!(
            "writable by {} — running it anyway under `trust = \"warn\"`",
            if group_w && world_w {
                "group and others"
            } else if group_w {
                "group"
            } else {
                "others"
            }
        ))
    } else {
        None
    };
    match trust {
        ExecTrust::Warn => match warn {
            Some(msg) => PermVerdict::Warn(msg),
            None => PermVerdict::Accept,
        },
        ExecTrust::NotWorldWritable => {
            if world_w {
                PermVerdict::Refuse(describe("others"))
            } else {
                PermVerdict::Accept
            }
        }
        ExecTrust::OwnerOnly => {
            if group_w || world_w {
                let who = if group_w && world_w {
                    "group or others"
                } else if group_w {
                    "group"
                } else {
                    "others"
                };
                PermVerdict::Refuse(describe(who))
            } else {
                PermVerdict::Accept
            }
        }
    }
}

/// Read the mode word and judge it. `None` from [`script_mode`] means "no answer here" —
/// no mode word on this platform, or the file is absent and the reader that follows owns
/// that error — and accepts.
fn permission_verdict(
    path: &std::path::Path,
    trust: crate::config::ExecTrust,
) -> Result<PermVerdict> {
    match script_mode(path)? {
        Some(mode) => Ok(judge_script_perms(mode, trust)),
        None => Ok(PermVerdict::Accept),
    }
}

#[cfg(unix)]
fn script_mode(path: &std::path::Path) -> Result<Option<u32>> {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(path) {
        Ok(meta) => Ok(Some(meta.permissions().mode())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::Validation(format!(
            "cannot read the permissions of `exec:` script {}: {}",
            path.display(),
            e
        ))),
    }
}

/// There is no mode word to read on Windows without an ACL walk std cannot do; the gate is
/// unix-shaped by design, and this arm exists so the *same* enforcement point runs everywhere.
#[cfg(not(unix))]
fn script_mode(_path: &std::path::Path) -> Result<Option<u32>> {
    Ok(None)
}

#[cfg(test)]
mod exec_guard_tests {
    use super::*;

    struct Fx {
        _tmp: tempfile::TempDir,
        execs: Execs<'static>,
        _config: std::sync::Arc<crate::config::Config>,
        _registry: std::sync::Arc<crate::backends::BackendRegistry>,
        _journal: std::sync::Arc<tokio::sync::Mutex<crate::core::Journal>>,
        _reaping: std::sync::Arc<crate::app::sync::guard::Reaping>,
    }

    fn fx() -> Fx {
        let tmp = tempfile::tempdir().unwrap();
        let config = std::sync::Arc::new(crate::config::Config::default());
        let executor = crate::core::CommandExecutor::new(true, false);
        let registry = std::sync::Arc::new(crate::backends::BackendRegistry::new());
        let journal = std::sync::Arc::new(tokio::sync::Mutex::new(
            crate::core::Journal::at(tmp.path().join("journal.jsonl")).unwrap(),
        ));
        let reaping = std::sync::Arc::new(crate::app::sync::guard::Reaping::new());
        // Leaked so the borrows outlive the struct; a test process does not care.
        let execs: Execs<'static> = Execs {
            config: Box::leak(Box::new(config.clone())),
            executor: Box::leak(Box::new(executor)),
            registry: Box::leak(Box::new(registry.clone())),
            journal: Box::leak(Box::new(journal.clone())),
            reaping: Box::leak(Box::new(reaping.clone())),
        };
        Fx {
            _tmp: tmp,
            execs,
            _config: config,
            _registry: registry,
            _journal: journal,
            _reaping: reaping,
        }
    }

    /// A script the plan promised but the machine no longer has is a refusal, not an empty
    /// body: `sh -c ""` exits 0, and the run would be recorded as a success against the old
    /// hash with nothing having happened.
    #[tokio::test]
    async fn a_script_that_vanished_between_plan_and_apply_is_an_error_not_a_success() {
        let f = fx();
        let gone = f._tmp.path().join("departed.sh");
        let err = f
            .execs
            .run_exec_script(&gone)
            .await
            .expect_err("a missing script must refuse");
        let msg = err.to_string();
        assert!(msg.contains("cannot read"), "{msg}");
        assert!(
            msg.contains("departed.sh"),
            "the error names the file: {msg}"
        );
    }

    use crate::app::apply::execs::judge_script_perms;
    use crate::config::ExecTrust;

    /// The table of mode bits × trust levels. Every platform runs this, because the judge is
    /// pure; only the file-reading half is unix-gated.
    #[test]
    fn the_trust_levels_judge_the_same_bits_differently() {
        use PermVerdict::*;
        let plain = 0o100_600;
        for mode in [plain, 0o100_400, 0o100_700] {
            for trust in [
                ExecTrust::OwnerOnly,
                ExecTrust::NotWorldWritable,
                ExecTrust::Warn,
            ] {
                assert!(
                    matches!(judge_script_perms(mode, trust), Accept),
                    "{trust:?} must accept {mode:o}"
                );
            }
        }
        // Group write: tolerated at the default level, refused by `owner-only`, warned under
        // `warn`.
        assert!(matches!(
            judge_script_perms(0o100_660, ExecTrust::OwnerOnly),
            Refuse(_)
        ));
        assert!(matches!(
            judge_script_perms(0o100_660, ExecTrust::NotWorldWritable),
            Accept
        ));
        assert!(matches!(
            judge_script_perms(0o100_660, ExecTrust::Warn),
            Warn(_)
        ));
        // World write: refused everywhere but `warn`.
        for trust in [ExecTrust::OwnerOnly, ExecTrust::NotWorldWritable] {
            assert!(
                matches!(judge_script_perms(0o100_666, trust), Refuse(_)),
                "{trust:?} must refuse a world-writable script"
            );
        }
        assert!(matches!(
            judge_script_perms(0o100_666, ExecTrust::Warn),
            Warn(_)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn the_gate_reads_real_files() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let script = tmp.path().join("drop.sh");
        std::fs::write(&script, "#!/bin/sh\ntrue\n").unwrap();
        // The default level accepts an ordinary private checkout and refuses the dropped-in
        // world-writable one.
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(matches!(
            permission_verdict(&script, ExecTrust::default()).unwrap(),
            PermVerdict::Accept
        ));
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o666)).unwrap();
        match permission_verdict(&script, ExecTrust::default()).unwrap() {
            PermVerdict::Refuse(msg) => {
                assert!(msg.contains("not-world-writable"), "{msg}");
            }
            other => panic!("world-write must refuse under the default, got {other:?}"),
        }
        // A missing file is the reader's error to give, not this gate's.
        assert!(matches!(
            permission_verdict(&tmp.path().join("nope.sh"), ExecTrust::default()).unwrap(),
            PermVerdict::Accept
        ));
    }

    #[test]
    fn the_exec_table_parses_and_defaults_to_not_world_writable() {
        let c: crate::config::Config = toml::from_str("[exec]\ntrust = \"owner-only\"\n").unwrap();
        assert_eq!(c.exec.trust, ExecTrust::OwnerOnly);
        let c: crate::config::Config = toml::from_str("").unwrap();
        assert_eq!(c.exec.trust, ExecTrust::NotWorldWritable);
        let err = toml::from_str::<crate::config::Config>("[exec]\ntrust = \"yolo\"\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("yolo"), "{err}");
    }
}
