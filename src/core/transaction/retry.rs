//! One batch, phase by phase: who may run, what was there before, what the manager said, what
//! the journal and the hooks make of it, and what the caller is told.
//!
//! **A batch is five separable jobs and this used to be all of them in one 409-line method.**
//! That method is where "what is on the machine now" is decided, so a reader who could not hold
//! it in one pass could not answer the only question this file exists for.

use super::*;

/// Everything one batch needs, carried as one value.
///
/// Eight parameters in a fixed order, five of them `Arc`s the worker already owns: the shape
/// [`crate::app::machinery::Machinery`] exists for, applied to the one call site that had it.
pub struct BatchArgs {
    pub batch: Batch,
    pub registry: Arc<BackendRegistry>,
    pub journal: Arc<Mutex<Journal>>,
    pub config: TransactionConfig,
    /// The user's configuration, for the one question a failed install asks of it: was the
    /// version this batch could not get a version Shall recorded rather than one they typed?
    pub app_config: Arc<crate::config::Config>,
    pub reaped: Option<crate::app::sync::guard::Reaped>,
    pub hooks: Option<Arc<LuaHooks>>,
    pub cancel_token: CancellationToken,
}

/// When this batch started, as both the wall clock a report carries and the instant a duration
/// is measured from. One value, so no member can claim a start time its batch did not have.
#[derive(Clone, Copy)]
struct BatchClock {
    at: chrono::DateTime<chrono::Utc>,
    instant: Instant,
}

impl BatchClock {
    fn now() -> Self {
        Self {
            at: chrono::Utc::now(),
            instant: Instant::now(),
        }
    }
}

/// One package on the command line, with the node it reports to.
type Member = (NodeIndex, GraphAction, String);

/// What a batch has accumulated between one phase and the next.
#[derive(Default)]
struct BatchState {
    members: Vec<Member>,
    priors: Vec<Prior>,
    ids: Vec<String>,
    /// The positions in `members` that are still going to the manager.
    keep: Vec<usize>,
    refused: Vec<TaskResult>,
}

/// A node that never reached the manager, reported as itself.
fn stillborn(
    b_name: &str,
    at: chrono::DateTime<chrono::Utc>,
    idx: NodeIndex,
    name: String,
    prior: Prior,
    e: Error,
) -> TaskResult {
    TaskResult {
        node_index: idx,
        backend_name: b_name.to_string(),
        package_name: name,
        retries: 0,
        duration: Duration::ZERO,
        bytes_downloaded: 0,
        start_time: at,
        prior,
        batch_size: 1,
        result: Err(e),
    }
}

/// Which manager this batch runs, and in which direction. `None` for an empty batch, which has
/// nothing to ask anyone.
fn batch_direction(batch: &Batch) -> Option<(bool, String)> {
    match batch.first().map(|(_, a)| a) {
        Some(GraphAction::Install(s)) => Some((true, s.backend.clone())),
        Some(GraphAction::Remove { backend, .. }) => Some((false, backend.clone())),
        None => None,
    }
}

impl Transaction {
    /// One batch of one manager command, from validation to the `TaskResult`s the caller reads.
    pub(super) async fn execute_batch_with_retry(args: BatchArgs) -> Vec<TaskResult> {
        let BatchArgs {
            batch,
            registry,
            journal,
            config,
            app_config,
            reaped,
            hooks,
            cancel_token,
        } = args;

        let started = BatchClock::now();
        let Some((is_install, b_name)) = batch_direction(&batch) else {
            return Vec::new();
        };

        let mut state = BatchState::default();
        let Some(backend_cap) =
            Self::admit_members(&batch, &b_name, &started, &registry, &mut state)
        else {
            return state.refused;
        };

        // Read before anything is done to it, per package. Rollback compensates by putting
        // this back, and "what was there before" is unknowable once the command has run.
        // Skipped entirely when there is no rollback to feed. Concurrent, and cheap now that
        // one listing per manager serves every question in the run.
        state.priors = Self::prior_states(&backend_cap, &state.members, &config).await;

        // The WAL, per package and before the manager is invoked. Recovery depends on every
        // entry reaching disk first, and batching does not change that — `record_starts`
        // writes all of them and flushes once, so each is durable before this returns.
        if !Self::open_entries(&journal, &b_name, &started, &mut state).await {
            return state.refused;
        }

        if is_install {
            Self::fire_before_install(&hooks, &journal, &config, &b_name, &started, &mut state)
                .await;
        } else {
            state.keep.extend(0..state.members.len());
        }
        // Nothing to close on the way out: the only thing that removes a member from `keep` is
        // the `before_install` arm above, which closes that member's entry as it drops it. This
        // is the one early return in the window between opening the WAL entries and closing
        // them that is allowed to leave without touching them.
        if state.keep.is_empty() {
            return state.refused;
        }

        let batch_size = state.keep.len();
        let (specs, names) = Self::command_packages(&backend_cap, &b_name, is_install, &state);
        let run = BatchRun {
            specs: &specs,
            names: &names,
            backend_cap: &backend_cap,
            backend: &b_name,
            is_install,
            config: &config,
            reaped,
            cancel_token: &cancel_token,
        };
        let (per_member, cancelled, attempt) =
            Self::run_with_recovery(run, state.keep.len(), &config).await;

        // **A version the user never typed must not fail in the user's face unexplained.** The
        // command named one manager and up to `batch_size` packages, and the manager's complaint
        // quotes whichever of them it choked on - so each member is asked, and only the one whose
        // recorded pin appears in the text answers. Narrowing makes this MORE accurate, not less:
        // a member that failed on its own carries an error about itself.
        let per_member = Self::with_pin_advice(&app_config, &b_name, &state, per_member);

        // The journal, once, from the verdicts. Every path used to write its own copy of this
        // block; there is one now, which is the other half of what narrowing bought.
        Self::close_entries(&journal, &state, &per_member, cancelled).await;

        if is_install {
            Self::fire_after_install(&hooks, &state, &per_member, config.max_concurrent).await;
        }

        Self::batch_results(&started, &b_name, state, per_member, attempt, batch_size)
    }

    /// Who may reach the command line at all, and the `TaskResult`s for those who may not.
    ///
    /// The grammar checks what a *file* declares. A removal target comes from
    /// `registry.json`, which apt's post-invoke hook also writes, so it has not been through
    /// the grammar at all. A name that cannot be validated is refused on its own and never
    /// reaches the shared command line.
    fn admit_members(
        batch: &Batch,
        b_name: &str,
        started: &BatchClock,
        registry: &Arc<BackendRegistry>,
        state: &mut BatchState,
    ) -> Option<Arc<crate::core::BackendCapabilities>> {
        for (idx, action) in batch {
            let p_name = match action {
                GraphAction::Install(s) => s.name.clone(),
                GraphAction::Remove { name, .. } => name.clone(),
            };
            match crate::core::Validator::validate_package_name_for(&p_name, b_name) {
                Ok(()) => state.members.push((*idx, action.clone(), p_name)),
                Err(e) => state.refused.push(stillborn(
                    b_name,
                    started.at,
                    *idx,
                    p_name,
                    Prior::Unknown,
                    e,
                )),
            }
        }
        if state.members.is_empty() {
            return None;
        }

        let Some(cap) = registry.get(b_name) else {
            for (idx, _, name) in std::mem::take(&mut state.members) {
                state.refused.push(stillborn(
                    b_name,
                    started.at,
                    idx,
                    name,
                    Prior::Unknown,
                    Error::BackendNotFound(b_name.to_string()),
                ));
            }
            return None;
        };
        Some(cap)
    }

    /// Each member's state before this run touched it.
    async fn prior_states(
        backend_cap: &Arc<crate::core::BackendCapabilities>,
        members: &[Member],
        config: &TransactionConfig,
    ) -> Vec<Prior> {
        if !config.auto_rollback {
            return vec![Prior::Unknown; members.len()];
        }
        use futures::stream::StreamExt;
        // **`needless_collect` fires here and it is wrong.** `stream::iter` does take
        // any `IntoIterator`, but dropping the `collect` leaves the stream's items
        // borrowing `members` inside a future that has to be `'static` for the worker
        // pool, and rustc rejects the closure as "not general enough". The `Vec` is what
        // makes the names owned. One allocation per batch, deliberately.
        futures::stream::iter(members.iter().map(|m| m.2.clone()).collect::<Vec<_>>())
            .map(|name| {
                let backend_cap = backend_cap.clone();
                async move { Self::prior_state(&backend_cap, &name).await }
            })
            // **The width is the batch's concurrency limit, not the batch.** Every other
            // fan-out in this file is bounded by `max_concurrent`, because a batch of four
            // hundred members would otherwise open four hundred listings and four hundred
            // manager processes at once — on the one call whose whole purpose is to be cheap
            // and to run *before* anything is touched. `members.len().max(1)` was a bound on
            // nothing (`PLAN.md` #84).
            //
            // **`buffered`, deliberately, not `buffer_unordered`.** The result is indexed by
            // member position further down (`priors[i]`), so this is not a set of answers but
            // an answer per slot; the cap changes how many run at once, never which slot an
            // answer lands in. A future reader who "simplifies" this to `buffer_unordered`
            // gets a silent mis-index rather than a compile error.
            .buffered(config.max_concurrent.max(1))
            .collect()
            .await
    }

    /// Open one WAL entry per member, durably, before the manager is invoked.
    ///
    /// `false` means the batch may not run: a member whose start could not be recorded is
    /// refused rather than run unrecorded.
    async fn open_entries(
        journal: &Arc<Mutex<Journal>>,
        b_name: &str,
        started: &BatchClock,
        state: &mut BatchState,
    ) -> bool {
        let mut j = journal.lock().await;
        let actions: Vec<JournalAction> = state
            .members
            .iter()
            .map(|(_, action, _)| match action {
                GraphAction::Install(s) => JournalAction::Install(s.clone()),
                GraphAction::Remove { name, backend } => JournalAction::Remove {
                    name: name.clone(),
                    backend: backend.clone(),
                },
            })
            .collect();
        match j.record_starts(actions) {
            Ok(ids) => {
                state.ids = ids;
                true
            }
            Err(e) => {
                // **Nothing to close.** The batch is all-or-nothing: a failure here left
                // neither the file nor the in-memory map touched, so there is no entry stuck
                // at `InProgress` to send `heal` looking for a crash that never happened.
                drop(j);
                for i in 0..state.members.len() {
                    state.refused.push(stillborn(
                        b_name,
                        started.at,
                        state.members[i].0,
                        state.members[i].2.clone(),
                        state.priors[i].clone(),
                        Error::Journal(format!("WAL error: {}", e)),
                    ));
                }
                false
            }
        }
    }

    /// `before_install` fires per package, before any install attempt. A failing pre-hook takes
    /// that package out of the batch — its declared prerequisites were not met — and leaves the
    /// rest of the command alone.
    async fn fire_before_install(
        hooks: &Option<Arc<LuaHooks>>,
        journal: &Arc<Mutex<Journal>>,
        config: &TransactionConfig,
        b_name: &str,
        started: &BatchClock,
        state: &mut BatchState,
    ) {
        let Some(h) = hooks else {
            state.keep.extend(0..state.members.len());
            return;
        };
        // **Fanned out, because each hook is about a different package.** Both loops were
        // sequential and bracketed the one part that was made concurrent, so a batch of *k* paid
        // *2k* serial hook invocations around it. Each is a process spawn, an mlua eval or a Rhai
        // eval that can block on HTTP. `before_install` must precede *its own* package's install,
        // not everybody else's.
        use futures::stream::StreamExt;
        let asked: Vec<(usize, String)> = state
            .members
            .iter()
            .enumerate()
            .map(|(i, (_, _, name))| (i, name.clone()))
            .collect();
        let mut outcomes: Vec<(usize, std::result::Result<(), String>)> =
            futures::stream::iter(asked)
                .map(|(i, name)| {
                    let h = h.clone();
                    async move {
                        let r = h
                            .run_hook("before_install", &name)
                            .await
                            .map(|_| ())
                            .map_err(|e| format!("before_install hook failed: {}", e));
                        (i, r)
                    }
                })
                .buffer_unordered(config.max_concurrent.max(1))
                .collect()
                .await;
        // Declaration order restored before anything acts on it: `keep` indexes `members`, and
        // the batch below and every result it produces read it in order. A fan-out that returns
        // as it finishes must not decide that order.
        outcomes.sort_by_key(|(i, _)| *i);
        for (i, outcome) in outcomes {
            match outcome {
                Ok(()) => state.keep.push(i),
                Err(msg) => {
                    let (idx, _, name) = &state.members[i];
                    let mut j = journal.lock().await;
                    let _ = j.record_failure(&state.ids[i], &msg);
                    drop(j);
                    state.refused.push(stillborn(
                        b_name,
                        started.at,
                        *idx,
                        name.clone(),
                        state.priors[i].clone(),
                        Error::Transaction(msg),
                    ));
                }
            }
        }
    }

    /// The argv's contents: the specs that install, the names that remove.
    fn command_packages(
        backend_cap: &Arc<crate::core::BackendCapabilities>,
        b_name: &str,
        is_install: bool,
        state: &BatchState,
    ) -> (Vec<PackageSpec>, Vec<String>) {
        let specs: Vec<PackageSpec> = state
            .keep
            .iter()
            .filter_map(|&i| match &state.members[i].1 {
                GraphAction::Install(s) => Some(s.clone()),
                _ => None,
            })
            .collect();
        let names: Vec<String> = state
            .keep
            .iter()
            .map(|&i| state.members[i].2.clone())
            .collect();

        // **A compensating install carries a version nobody declared** (`Q53`). Rollback reads
        // what a package was on before the transaction touched it and puts that back, so a
        // version reaches this point on a manager the planner never got to vet — and on a
        // manager that cannot be asked for one there is nothing to send.
        //
        // Stripped and **named**, not refused: refusing would end a rollback with the package
        // uninstalled, which is worse than putting it back at what the manager offers and
        // saying which version could not be restored. The declared case is the one that gets
        // refused, and it is refused at plan time.
        //
        // Before this, `brew.rs` answered the same situation by building `pkg-a@1.0` — a
        // formula name that does not exist — so the rollback failed on a real Mac and passed in
        // every test, because a mock matches any string.
        // Only an install into a manager that cannot be asked for a version has anything to
        // strip. The plain `||` is what keeps this one branch rather than two.
        if !is_install
            || backend_cap
                .as_installable()
                .is_some_and(|i| i.pins_version())
        {
            return (specs, names);
        }
        let specs = specs
            .into_iter()
            .map(|mut spec| {
                match spec.options.one("version").map(str::to_string) {
                    Some(v) if crate::backends::concrete_version(&v) => {
                        warn!(
                            "{}:{} was on {} and `{}` cannot be asked for a version — \
                             putting it back at whatever `{}` offers.{}",
                            b_name,
                            spec.name,
                            v,
                            b_name,
                            b_name,
                            match crate::backends::capability::cannot_pin_reason(b_name) {
                                Some(why) => format!(" {}.", why),
                                None => String::new(),
                            }
                        );
                        spec.options.remove("version");
                    }
                    _ => {}
                }
                spec
            })
            .collect();
        (specs, names)
    }

    /// **One command, and then narrowing.** The retry loop lives in `run_one_command` so that a
    /// failed batch can be asked again over half of itself WITHOUT re-opening a WAL entry or
    /// firing `before_install` a second time. A narrowing is a retry with a shorter command
    /// line, and a retry has never done either.
    ///
    /// Returns one verdict per `keep` position, whether the batch was cancelled, and the
    /// attempt it took.
    async fn run_with_recovery(
        run: BatchRun<'_>,
        members: usize,
        config: &TransactionConfig,
    ) -> (Vec<std::result::Result<(), Error>>, bool, u32) {
        let outcome = run_one_command(run).await;
        let attempt = outcome.attempt();
        let mut cancelled = false;
        let per_member: Vec<std::result::Result<(), Error>> = match outcome {
            CommandOutcome::Done { .. } => vec![Ok(()); members],
            CommandOutcome::Cancelled { .. } => {
                cancelled = true;
                vec![Err(Error::Cancelled); members]
            }
            CommandOutcome::Failed { error, .. } => {
                if config
                    .batch_recovery
                    .narrows(&error, members, config.continue_past)
                {
                    narrow_batch(run).await
                } else {
                    vec![Err(error); members]
                }
            }
        };
        (per_member, cancelled, attempt)
    }

    /// Each failure's own pin advice, if the manager's complaint is about a version Shall
    /// recorded rather than one the user typed.
    fn with_pin_advice(
        app_config: &Arc<crate::config::Config>,
        b_name: &str,
        state: &BatchState,
        per_member: Vec<std::result::Result<(), Error>>,
    ) -> Vec<std::result::Result<(), Error>> {
        per_member
            .into_iter()
            .enumerate()
            .map(|(p, r)| match r {
                Err(e) => {
                    let text = e.to_string();
                    match crate::app::sync::pin_advice::on_install_failure(
                        app_config,
                        b_name,
                        &state.members[state.keep[p]].2,
                        &text,
                    ) {
                        Some(advice) => Err(e.with_note(advice)),
                        None => Err(e),
                    }
                }
                ok => ok,
            })
            .collect()
    }

    /// The WAL, once, from the verdicts.
    async fn close_entries(
        journal: &Arc<Mutex<Journal>>,
        state: &BatchState,
        per_member: &[std::result::Result<(), Error>],
        cancelled: bool,
    ) {
        let mut j = journal.lock().await;
        for (p, &i) in state.keep.iter().enumerate() {
            match &per_member[p] {
                Ok(()) => {
                    let _ = j.record_success(&state.ids[i]);
                }
                Err(e) => {
                    let _ = j.record_failure(&state.ids[i], &wal_failure_reason(cancelled, e));
                }
            }
        }
    }

    /// `after_install` fires once a package is physically installed, and only for the ones that
    /// were. Fanned out for the same reason as `before_install`: each is about one package, and
    /// a failure here is logged rather than acted on.
    async fn fire_after_install(
        hooks: &Option<Arc<LuaHooks>>,
        state: &BatchState,
        per_member: &[std::result::Result<(), Error>],
        max_concurrent: usize,
    ) {
        let Some(h) = hooks else {
            return;
        };
        use futures::stream::StreamExt;
        let asked: Vec<String> = state
            .keep
            .iter()
            .enumerate()
            .filter(|(p, _)| per_member[*p].is_ok())
            .map(|(_, &i)| state.members[i].2.clone())
            .collect();
        futures::stream::iter(asked)
            .map(|name| {
                let h = h.clone();
                async move {
                    if let Err(e) = h.run_hook("after_install", &name).await {
                        warn!("after_install hook for '{}' failed: {}", name, e);
                    }
                }
            })
            .buffer_unordered(max_concurrent.max(1))
            .collect::<Vec<()>>()
            .await;
    }

    /// What the caller is told: one row per member, refused and attempted alike.
    fn batch_results(
        started: &BatchClock,
        b_name: &str,
        state: BatchState,
        per_member: Vec<std::result::Result<(), Error>>,
        attempt: u32,
        batch_size: usize,
    ) -> Vec<TaskResult> {
        let BatchState {
            members,
            priors,
            keep,
            mut refused,
            ..
        } = state;
        refused.extend(keep.iter().enumerate().map(|(p, &i)| {
            let (idx, _, name) = &members[i];
            TaskResult {
                node_index: *idx,
                backend_name: b_name.to_string(),
                package_name: name.clone(),
                retries: retries_behind(attempt),
                duration: started.instant.elapsed(),
                bytes_downloaded: 0,
                start_time: started.at,
                prior: priors[i].clone(),
                batch_size,
                result: per_member[p].clone(),
            }
        }));
        refused
    }
}
