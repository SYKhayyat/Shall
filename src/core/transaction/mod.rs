use super::batch::{narrow_batch, run_one_command, BatchRecovery, BatchRun, CommandOutcome};
use crate::app::diagnostics::FailureDiagnosticEngine;
use crate::app::LuaHooks;
use crate::backends::BackendRegistry;
use crate::core::journal::JournalAction;
use crate::core::{Error, Journal, PackageSpec, Result, Retryability};
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use petgraph::Direction;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace, warn};

mod retry;
mod rollback;

use retry::BatchArgs;
pub use rollback::{plan_undo, RollbackPlan, Undo, UndoFacts};

/// How far a transaction carries on past a node that failed.
///
/// **Three values and not two booleans**, because they are ordered and exactly one holds: a run
/// that carries on past everything is not also a run that carries on past some of it, and a pair
/// of flags can express that contradiction. II.29 - a kind is a type, and every dispatch over it
/// is exhaustive.
///
/// A node whose *dependency* failed is never attempted under any of the three; it is reported as
/// skipped, naming the one that stopped it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuePast {
    /// Nothing. The first failed node ends the transaction and the rest is never attempted.
    ///
    /// Still the right answer for a failure that says the plan itself is wrong: a plan is one
    /// change to one machine, so a member that cannot work makes the whole plan suspect and the
    /// rest of it must not be half-applied.
    Nothing,
    /// A failure Shall itself classified as passing - `Transient`, or the `Exhausted` that a
    /// transient becomes once the retry loop has falsified it.
    ///
    /// **The category `Y15` did not have.** That ruling drew its line between a backend this
    /// machine does not have (skipped - the config is portable, not broken) and a package that
    /// failed (fail the run), because in August every failure of this third kind arrived as
    /// `Retryability::Unknown` and there was nothing to key on. There is now: a rotated registry
    /// key or an index that will not verify is neither the config's fault nor fixable by editing
    /// the line, and one such line must not strand the two hundred beside it any more than one
    /// `apt:` line may strand twenty `winget:` ones.
    ///
    /// Continuing is still not succeeding (`G1`): the run finishes what it can, reports what it
    /// did not, and exits non-zero.
    ClassifiedPassing,
    /// Any failure at all - `--keep-going`, and recovery.
    ///
    /// Recovery is the shape this was built for: each entry is a separate piece of interrupted
    /// work left by a run that already died, and one that cannot be finished is not a reason to
    /// leave the others unfinished.
    AnyFailure,
}

impl ContinuePast {
    /// Whether a round of failures may be carried past.
    ///
    /// Named rather than inlined so it can be asserted. The decision is three lines inside a
    /// two-hundred-line scheduler loop, and a test that had to build a DAG and time a wave to
    /// reach it would be measuring the scheduler instead of the rule.
    ///
    /// `every_failure_passing` is about the whole round and not one node: a batch that failed
    /// with one `Permanent` among the transients is stopped by the `Permanent`, because that is
    /// the one saying the plan is wrong.
    pub fn carries_on(self, every_failure_passing: bool) -> bool {
        match self {
            Self::AnyFailure => true,
            Self::ClassifiedPassing => every_failure_passing,
            Self::Nothing => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransactionConfig {
    pub max_concurrent: usize,
    pub node_timeout: Duration,
    pub total_timeout: Duration,
    pub max_retries: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub auto_rollback: bool,
    /// How far this transaction carries on past a node that failed.
    pub continue_past: ContinuePast,
    /// What a batch does after its command fails for a passing reason.
    pub batch_recovery: BatchRecovery,
    /// Remove also destroys configuration (`[remove] purge`, or `uninstall --purge`). A
    /// backend that draws no such distinction removes as usual — the decision cannot be
    /// per-package because a removal happens after the line that carried it is gone.
    pub purge: bool,
    /// How long to wait for another package manager that holds its own lock. Zero does not
    /// wait. See `manager_lock_wait_secs` in the config for why this is not a backoff.
    pub manager_lock_wait: Duration,
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self::patient()
    }
}

impl TransactionConfig {
    /// The defaults. `sync` overrides `max_concurrent` from `max_parallel`; this is what every
    /// other constructor gets, so it is the machine's parallelism rather than the number 4.
    pub fn patient() -> Self {
        Self {
            max_concurrent: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
            node_timeout: Duration::from_secs(300),
            total_timeout: Duration::from_secs(3600),
            max_retries: 3,
            initial_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(30),
            auto_rollback: true,
            continue_past: ContinuePast::Nothing,
            batch_recovery: BatchRecovery::Off,
            purge: false,
            manager_lock_wait: Duration::from_secs(
                crate::config::declaration::default_manager_lock_wait_secs(),
            ),
        }
    }

    /// The settings a run's `Config` decides, in one place.
    ///
    /// These were three ad-hoc reads at the one call site, and the comment above the first of
    /// them recorded what that costs: `max_concurrent` had been left at the `patient()`
    /// default, which "silently narrows the setting's reach to `search` alone". A named
    /// constructor is where the fourth one goes instead of becoming a fourth ad-hoc line.
    pub fn from_config(config: &crate::config::Config) -> Self {
        Self {
            max_concurrent: config.max_parallel.max(1),
            purge: config.remove.purge || config.purge_this_run,
            // `--keep-going` outranks the file key rather than combining with it: the flag is
            // a per-run instruction from somebody at a keyboard, and the key is what the
            // machine does when nobody said.
            continue_past: if config.keep_going_this_run {
                ContinuePast::AnyFailure
            } else if config.sync.continue_past_transient {
                ContinuePast::ClassifiedPassing
            } else {
                ContinuePast::Nothing
            },
            batch_recovery: config.sync.batch_recovery,
            manager_lock_wait: Duration::from_secs(config.manager_lock_wait_secs),
            // The per-package budget, and the number `patient()` hard-codes is this key's
            // default, so the default is unchanged and the key has a lever. `.max(1)` is a floor
            // rather than politeness: zero would kill every manager command on arrival.
            node_timeout: Duration::from_secs(config.manager_command_timeout_secs.max(1)),
            ..Self::patient()
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GraphAction {
    Install(PackageSpec),
    Remove { name: String, backend: String },
}

/// What a node's target looked like *before* that node ran.
///
/// Rollback compensates by putting this back, so it has to be a fact rather than an
/// assumption. Compensating an `Install` with a removal is right only when the package was
/// absent to begin with — and it often is not: a `@version=` or `@channel=` change schedules an
/// `Install` node for a package that is already there, so removing it uninstalls software the
/// user had instead of reverting a version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prior {
    /// The package was not installed before this node ran.
    Absent,
    /// It was installed, at this version when the manager reported one.
    Present(Option<String>),
    /// The manager could not be asked, or has no query capability. Nothing is inferred from
    /// this — "I could not tell" is not "it was not there".
    Unknown,
}

/// The nodes one manager command covers, paired with what each of them asks for.
type Batch = Vec<(NodeIndex, GraphAction)>;

#[derive(Debug, Clone)]
pub struct TaskResult {
    pub node_index: NodeIndex,
    pub backend_name: String,
    pub package_name: String,
    /// How many times this node was *retried* — 0 on a first-try success. Named for what it
    /// holds: it fed a parameter called `retry_count` while being called `attempt`, so the
    /// arithmetic below reads like an off-by-one to everyone who checks it.
    pub retries: u32,
    pub duration: Duration,
    pub bytes_downloaded: u64,
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// What this node's target looked like before it ran. Only read by `rollback`.
    pub prior: Prior,
    /// How many packages the single manager command that covered this one carried.
    ///
    /// `1` unless this node was batched. Reported, because six packages sharing one `apt
    /// install` produce six identical durations — and six identical durations under a heading
    /// that says "Parallel Task Breakdown" is exactly how a serialised run read as a parallel
    /// one for as long as it did. Now they are identical for a reason the output states.
    pub batch_size: usize,
    pub result: Result<()>,
}

pub struct Transaction {
    pub graph: StableDiGraph<GraphAction, ()>,
    registry: Arc<BackendRegistry>,
    journal: Arc<Mutex<Journal>>,
    diagnostics: Arc<FailureDiagnosticEngine>,
    config: TransactionConfig,
    /// The user's configuration, for the removal guard. A rollback's compensating removals are
    /// issued here, at execution time, and never pass the plan-time gate in `sync` — so this is
    /// the only place they can be checked, and a guard on one path is a guard on nothing.
    app_config: Arc<crate::config::Config>,
    /// Optional lifecycle hooks. When set, `before_install` fires for every member of a batch
    /// before the batch runs and `after_install` for every member that installed, both fanned
    /// out at `max_concurrent` — a hook is about its own package, not about the batch.
    ///
    /// A failing `before_install` takes that one package out of the batch and leaves the rest
    /// of the command alone; a failing `after_install` is logged and undoes nothing, because
    /// rolling back a healthy package over a cosmetic hook error is more surprising than the
    /// failure.
    hooks: Option<Arc<LuaHooks>>,
    completed_indices: HashSet<NodeIndex>,
    /// Each finished node with what its target looked like before it ran. Rollback walks this
    /// backwards, and cannot compensate correctly without the second half.
    pub history: Vec<(NodeIndex, Prior)>,
    cancellation_token: CancellationToken,
    /// Proof that the plan this graph came from passed the removal guard.
    ///
    /// `None` until [`Transaction::guarded_by`] is called, and a graph carrying a removal node
    /// **refuses to execute without it**. The guard runs at plan time, in the engine, over the
    /// whole plan at once — which is where it has to run, because `max_removals` is a ceiling
    /// over a plan and cannot be checked one argv at a time. What was missing was any way for
    /// the executor to know it had happened; a plan built by some other path and handed
    /// straight here would have removed packages with nothing in between.
    ///
    /// This one is a runtime refusal rather than a compile error, and that is worth saying
    /// plainly: making it a compile error would mean typing the graph itself by whether it
    /// contains a removal, which is a larger change than this finding earns. The five effectors
    /// **are** compile-enforced; this is the seam that hands them their token.
    reaped: Option<crate::app::sync::guard::Reaped>,
    /// **What this plan intends the machine to end up holding**, as `backend:name` — the
    /// `Install` nodes of the graph being executed.
    ///
    /// **Rollback consults it in both directions, and that is one rule, not two** (`U41`,
    /// amended 2026-08-09). *Rollback does not undo work that moved the machine toward the
    /// declared state.*
    ///
    /// - **An install that succeeded, of something still declared,** is not failed work — it is
    ///   the goal, reached early. `Prior::Absent` says the package was not here before this run;
    ///   it does not say nobody wants it, and the manifest holds the second fact. Removing it
    ///   hands the next `sync` the same work to do again.
    /// - **A removal that succeeded, of something still undeclared,** is the same event seen
    ///   from the other side. The fact that authorised the removal — nothing declares this — is
    ///   still true when the rollback fires, and it is knowable the same way it was knowable
    ///   then: the package is not in this set. Re-installing it un-converges exactly as
    ///   symmetrically.
    ///
    /// **What is lost by the second half, stated plainly:** a package the user had, that this
    /// run removed, stays removed after a failed transaction. The durable put-it-back is
    /// generations and snapshots, which is what they are for; the WAL records the removal but
    /// not the version, so a `Prior` that outlived the process would be the alternative and it
    /// is deferred, not rejected (`U41`).
    ///
    /// `None` for a transaction that is not reconciling against a manifest — see
    /// [`GuardScope::reconciles`](crate::app::sync::guard::GuardScope::reconciles). There the
    /// old behaviour is right in both arms: a `rebuild`'s removal phase is half of a reinstall
    /// of declared packages, and a hand-typed `uninstall` was not derived from anything.
    pub declared: Option<Arc<std::collections::HashSet<String>>>,
    /// What the last execution's scheduling actually looked like — packages, graph depth, and
    /// the number of times the engine went idle and handed out more work.
    ///
    /// Recorded rather than only logged, because the shape rule for `Mutating` is otherwise
    /// asserted by reading a `warn!` line. A warning nothing can read is the shape of a gate
    /// that cannot fail, which is what the rule was written to replace.
    pub last_scheduling: Option<Scheduling>,
}

/// How serially one execution actually ran, against how serial its graph forced it to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scheduling {
    /// Nodes this run had left to do — the whole graph, less anything a resume had finished.
    pub packages: usize,
    /// The longest dependency chain among them, which is the fewest waves any correct scheduler
    /// could take.
    pub depth: usize,
    /// Times the engine had nothing in flight and then handed work out. At most `depth` for a
    /// scheduler that dispatches every ready node the moment it is ready; `packages` for a
    /// serial loop. Below `depth` whenever independent chains overlap, which is why the rule is
    /// an inequality and not an equality.
    pub waves: usize,
}

/// The scheduler's own bookkeeping between passes: what is ready, what is in flight, what has
/// been answered, and how many waves the engine has gone idle for.
struct RunState {
    in_progress: HashSet<NodeIndex>,
    pending_deps: HashMap<NodeIndex, usize>,
    ready: Vec<NodeIndex>,
    results: Vec<TaskResult>,
    waves: usize,
    /// The longest remaining dependency chain, which is the fewest waves a correct scheduler can
    /// take. Taken before the run, because dispatching is what changes `completed_indices`.
    depth: usize,
    remaining: usize,
}

impl RunState {
    fn new(tx: &Transaction) -> Self {
        let total = tx.graph.node_count();
        // The shape budget for `Mutating`, which the fan-out rule in `latency.rs` cannot
        // express: both quantities come from this graph, so the engine reports its own shape
        // rather than being inspected from a layer that is handed a subcommand name and nothing
        // else.
        let depth = tx.critical_path_depth();
        // How many unfinished dependencies each node still has. Decremented as they finish,
        // instead of rescanning every node and every incoming edge on every pass — which was
        // O(V·(V+E)) over a run, or ~100k redundant edge checks for 300 packages, and it only
        // gets worse as the batching makes the graph wide enough to notice.
        let pending_deps: HashMap<NodeIndex, usize> = tx
            .graph
            .node_indices()
            .map(|idx| {
                let n = tx
                    .graph
                    .neighbors_directed(idx, Direction::Incoming)
                    .filter(|dep| !tx.completed_indices.contains(dep))
                    .count();
                (idx, n)
            })
            .collect();
        let mut ready: Vec<NodeIndex> = pending_deps
            .iter()
            .filter(|(idx, n)| **n == 0 && !tx.completed_indices.contains(idx))
            .map(|(idx, _)| *idx)
            .collect();
        // Node order, not hash order, so a plan runs the same way twice.
        ready.sort();
        Self {
            in_progress: HashSet::new(),
            pending_deps,
            ready,
            results: Vec::new(),
            waves: 0,
            depth,
            remaining: total - tx.completed_indices.len(),
        }
    }
}

impl Transaction {
    pub fn new(
        graph: StableDiGraph<GraphAction, ()>,
        registry: Arc<BackendRegistry>,
        journal: Arc<Mutex<Journal>>,
        diagnostics: Arc<FailureDiagnosticEngine>,
        app_config: Arc<crate::config::Config>,
    ) -> Self {
        Self::with_config(
            graph,
            registry,
            journal,
            diagnostics,
            app_config,
            TransactionConfig::default(),
        )
    }

    pub fn with_config(
        graph: StableDiGraph<GraphAction, ()>,
        registry: Arc<BackendRegistry>,
        journal: Arc<Mutex<Journal>>,
        diagnostics: Arc<FailureDiagnosticEngine>,
        app_config: Arc<crate::config::Config>,
        config: TransactionConfig,
    ) -> Self {
        Self {
            graph,
            registry,
            journal,
            diagnostics,
            config,
            app_config,
            reaped: None,
            declared: None,
            hooks: None,
            completed_indices: HashSet::new(),
            history: Vec::new(),
            cancellation_token: CancellationToken::new(),
            last_scheduling: None,
        }
    }

    /// Packages with no configured hook (and no `*` wildcard) incur only a cheap map
    /// lookup, so this is safe to always set.
    pub fn with_hooks(mut self, hooks: Arc<LuaHooks>) -> Self {
        self.hooks = Some(hooks);
        self
    }

    /// Hand the executor proof that this plan's removals passed the guard.
    ///
    /// Required before executing any graph that contains a `Remove` node — see the `reaped`
    /// field. A graph of pure installs needs nothing, which is why this is a builder step
    /// rather than a constructor argument: an install-only plan should not have to produce a
    /// removal authorisation it has no removals for.
    pub fn guarded_by(mut self, reaped: crate::app::sync::guard::Reaped) -> Self {
        self.reaped = Some(reaped);
        self
    }

    /// Tell rollback what the manifest still asks for, as `backend:name`.
    ///
    /// See the `declared` field. Without it, rollback compensates work that succeeded and is
    /// still wanted, and `auto_rollback: true` — the default at `transaction.rs` — becomes an
    /// anti-convergent step. `heal`, whose entire job is the same failure shape, sets
    /// `auto_rollback: false`; nothing explained the split, and this is what it was standing in
    /// for.
    pub fn reconciling(mut self, declared: Arc<std::collections::HashSet<String>>) -> Self {
        self.declared = Some(declared);
        self
    }

    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    /// Does this plan intend the machine to end up holding this package?
    ///
    /// `None` when the run is not reconciling against a manifest and the question has no
    /// answer. **The whole of `U41` is that both rollback arms ask this one question**: the
    /// install arm skips its removal on `Some(true)`, the removal arm skips its reinstate on
    /// `Some(false)`, and neither does anything on `None`. Written as a function rather than
    /// twice inline so the symmetry is a fact about the code and not about two comments.
    pub fn plan_intends_present(&self, backend: &str, name: &str) -> Option<bool> {
        self.declared
            .as_ref()
            .map(|d| d.contains(&format!("{}:{}", backend, name)))
    }

    /// The WAL entries that were already open before this run started.
    ///
    /// **Snapshotted so that [`close_stranded`](Self::close_stranded) can tell the two kinds of
    /// open entry apart.** An entry left open by an *earlier* run is the record that a process
    /// died holding it, and it is the only thing that tells `heal` a real crash happened —
    /// closing it would erase the recovery state this log exists to keep.
    async fn open_before_this_run(&self) -> std::collections::HashSet<String> {
        self.journal
            .lock()
            .await
            .interrupted_actions()
            .into_iter()
            .map(|e| e.id)
            .collect()
    }

    /// Close every entry *this* run opened and then abandoned.
    ///
    /// **A batch that is aborted never reaches either of the calls that close its entry.** Both
    /// ways out of a failed run kill their workers — `abort_all` on the first failure, and the
    /// `JoinSet` being dropped on the way out — so a task that had called `record_start` and was
    /// still inside the manager's command dies with its entry `InProgress`.
    ///
    /// That state means "a process died holding this". Left behind by a run Shall itself
    /// stopped, it makes [`Journal::needs_recovery`] answer yes for ever and sends `heal`
    /// looking for a crash that never happened — the same class of defect as reporting that
    /// nothing is wrong, pointing the other way. Shall aborted these and knows it did, so it
    /// says so.
    ///
    /// Measured: the macOS nightly of 2026-08-14 ran an unrefused `purge-undeclared` over 276
    /// removals, `gem:logger` failed, `continue_past` was `Nothing`, and the rollback aborted
    /// every other manager's batch mid-command. The harness reported *"22 operation(s) are
    /// still open in the write-ahead log and nothing crashed"*.
    async fn close_stranded(&self, open_before: &std::collections::HashSet<String>, why: &Error) {
        let mut j = self.journal.lock().await;
        let stranded: Vec<String> = j
            .interrupted_actions()
            .into_iter()
            .map(|e| e.id)
            .filter(|id| !open_before.contains(id))
            .collect();
        if stranded.is_empty() {
            return;
        }
        let reason = format!("abandoned when the run stopped: {}", why);
        warn!(
            "{} operation(s) were still open when the run stopped; closing them as abandoned \
             rather than leaving them to read as a crash",
            stranded.len()
        );
        for id in stranded {
            // **Abandoned, not failed** (Q33). These entries never reached an outcome — the
            // batch they belonged to was aborted before it joined — so closing them as Failed
            // claimed an answer nobody has, and `heal`, which reads InProgress and Abandoned
            // as the interrupted set, walked past installs that may have half-run. This is
            // exactly the state the recovery sweep exists to finish.
            let _ = j.record_abandoned(&id, &reason);
        }
    }

    pub async fn execute_with_telemetry(&mut self) -> Result<Vec<TaskResult>> {
        let total_timeout = self.config.total_timeout;
        let start_time = Instant::now();

        info!(
            "Initializing parallel execution for {} nodes.",
            self.graph.node_count()
        );

        // Taken before any work, and read on both failing paths below. One place, because the
        // two ways a run can strand an entry are one bug.
        let open_before = self.open_before_this_run().await;

        let outcome = match tokio::time::timeout(total_timeout, self.execute_internal()).await {
            Ok(Ok(results)) => {
                debug!("DAG closure reached in {:?}", start_time.elapsed());
                Ok(results)
            }
            Ok(Err(e)) => {
                self.close_stranded(&open_before, &e).await;
                Err(e)
            }
            Err(_) => {
                error!(
                    "CRITICAL FAILURE - Global timeout of {:?} reached.",
                    total_timeout
                );
                self.cancellation_token.cancel();
                if self.config.auto_rollback {
                    if let Err(e) = self.rollback().await {
                        error!("{}", e);
                    }
                }
                let e =
                    Error::Transaction(format!("Transaction timed out after {:?}", total_timeout));
                self.close_stranded(&open_before, &e).await;
                Err(e)
            }
        };

        // The run is over, so there is no next wave whose opening would carry the last wave's
        // completions down with it. Every arm above reaches here, including the two that
        // closed stranded entries, because an entry closed and not written still reads as a
        // crash to the run after this one.
        let _ = self.journal.lock().await.flush();

        outcome
    }

    pub async fn execute(&mut self) -> Result<()> {
        self.execute_with_telemetry().await.map(|_| ())
    }

    /// The most waves this plan can honestly take: its longest remaining dependency chain.
    ///
    /// Measured over the nodes still to run, so a resumed transaction is judged on the work it
    /// has left rather than on work somebody else already did. A cycle has no depth and the
    /// planner rejects one long before here; returning zero switches the check off rather than
    /// inventing a number for a plan that cannot be scheduled at all.
    fn critical_path_depth(&self) -> usize {
        let Ok(order) = petgraph::algo::toposort(&self.graph, None) else {
            return 0;
        };
        let mut level: HashMap<NodeIndex, usize> = HashMap::new();
        let mut deepest = 0;
        for idx in order
            .into_iter()
            .filter(|i| !self.completed_indices.contains(i))
        {
            let depth = self
                .graph
                .neighbors_directed(idx, Direction::Incoming)
                .filter_map(|parent| level.get(&parent).copied())
                .max()
                .map_or(1, |deepest_parent| deepest_parent + 1);
            level.insert(idx, depth);
            deepest = deepest.max(depth);
        }
        deepest
    }

    async fn execute_internal(&mut self) -> Result<Vec<TaskResult>> {
        let mut run = RunState::new(self);
        let mut worker_pool = JoinSet::new();
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent));

        while self.completed_indices.len() < self.graph.node_count() {
            if self.cancellation_token.is_cancelled() {
                Self::quiesce_workers(&mut worker_pool).await;
                self.roll_back_quietly().await;
                return Err(Error::Transaction("Transaction cancelled.".into()));
            }

            self.dispatch_ready(&mut run, &semaphore, &mut worker_pool);

            let Some(finished_task) = worker_pool.join_next().await else {
                // **Nothing was joined, which can only mean the pool was already empty.**
                // `join_next` answers `None` for an empty `JoinSet` and for nothing else, so
                // reaching here says every task ever dispatched has been joined and accounted
                // for — every node of them removed from `in_progress` or returned on — and that
                // this pass dispatched nothing to replace them. The loop's own condition says
                // there is still work. That is a graph no scheduler can advance.
                //
                // This used to guard the report with `in_progress.is_empty() &&
                // completed < total`. Both are implied here and neither can be false, so the
                // guard could not change the outcome; what it could do is make the report
                // conditional on a fact nothing establishes, which is how a stall becomes an
                // infinite loop instead of a message.
                return Err(Error::Transaction(
                    "DAG Logic Stall: Cycle detected in closure.".into(),
                ));
            };
            let results =
                finished_task.map_err(|e| Error::Transaction(format!("Worker Panic: {}", e)))?;

            if let Some(final_err) = self.absorb_results(results, &mut run) {
                Self::quiesce_workers(&mut worker_pool).await;
                self.roll_back_quietly().await;
                return Err(final_err);
            }
        }

        self.report_scheduling(&run);
        Ok(run.results)
    }

    /// Undo what this run did, and never let the undoing's failure hide the failure that
    /// caused it. The caller has already quiesced its workers and already decided to stop; a
    /// rollback that returns an error is logged and not returned, because the error the user
    /// needs is the one that stopped the run.
    async fn roll_back_quietly(&mut self) {
        if !self.config.auto_rollback {
            return;
        }
        info!("rolling back");
        if let Err(e) = self.rollback().await {
            error!("{}", e);
        }
    }

    /// Hand out every batch that is ready now, and count the wave if the engine had gone idle.
    ///
    /// **A wave is work handed out after the engine went quiet — not every pass that
    /// dispatched something.** Counting passes looks equivalent and is not: two independent
    /// chains finishing at different moments produce a pass per completion, which is the
    /// scheduler dispatching *eagerly*, and a rule that counted those would fail the runs it
    /// exists to reward. What cannot happen in a correct run is going idle more times than the
    /// graph has levels, because each idle restart means an entire generation finished before
    /// the next was handed out.
    fn dispatch_ready(
        &self,
        run: &mut RunState,
        semaphore: &Arc<Semaphore>,
        worker_pool: &mut JoinSet<Vec<TaskResult>>,
    ) {
        // One package per command under `--keep-going`, so a name no repository carries cannot
        // take the installable packages beside it down with it.
        //
        // **`ClassifiedPassing` keeps the batch here, and `BatchRecovery` is what comes back
        // for its members afterwards.** `G1`'s argument for one-package-per-command is about a
        // bad NAME, a fact about one member, so the batch must come apart before the good
        // members can be told from it - but paying that on EVERY command to be ready for the
        // failures is what makes `--keep-going` expensive. Splitting after a failure costs the
        // same commands only when something actually failed, and `M3` bisects rather than
        // splitting flat, so it costs far fewer of them.
        let max_batch = match self.config.continue_past {
            ContinuePast::AnyFailure => 1,
            ContinuePast::Nothing | ContinuePast::ClassifiedPassing => Self::MAX_BATCH,
        };
        let dispatching = std::mem::take(&mut run.ready);
        if !dispatching.is_empty() && run.in_progress.is_empty() {
            run.waves += 1;
        }
        // **An empty batch is not work, and dispatching one looks exactly like progress.**
        // `execute_batch_with_retry` returns at once for a batch with no members, so the join
        // below succeeds, completes nothing, and the loop comes round to dispatch another — for
        // ever, against a `total_timeout` measured in hours. `batches` cannot produce one; the
        // filter is what stops it *mattering* whether that stays true, because with nothing
        // dispatched the pass falls through to the stall report and says so in milliseconds.
        // The sweep found this the expensive way: the mutation that makes `batches` answer one
        // empty batch was reported as a two-hour timeout in three consecutive nightlies.
        for batch in Self::batches(&self.graph, dispatching, max_batch)
            .into_iter()
            .filter(|b| !b.is_empty())
        {
            run.in_progress.extend(batch.iter().map(|(idx, _)| *idx));

            // **The permit is taken inside the worker, not here.** Acquired on this side it was
            // a `await` on the coordinator's own loop, so the concurrency ceiling throttled the
            // scheduler as well as the work: while a permit was unavailable the loop could not
            // dispatch anything, could not join a finished task, and therefore could not record
            // the completion that would release the permit. Every finished batch's bookkeeping
            // waited on the next permit (#18). Moving it one line in is what makes the ceiling
            // a ceiling on *work*: the task is already spawned and counted in `in_progress`,
            // and the loop goes straight on to `join_next`, so a completion is always recorded
            // promptly. The limit is unchanged — a task without a permit runs nothing.
            let permit_source = semaphore.clone();
            let registry = self.registry.clone();
            let journal = self.journal.clone();
            let cancel_token = self.cancellation_token.clone();
            let config = self.config.clone();
            let app_config = self.app_config.clone();
            let reaped = self.reaped;
            let hooks = self.hooks.clone();

            worker_pool.spawn(async move {
                let _permit_holder = match permit_source.acquire_owned().await {
                    Ok(p) => p,
                    // A closed semaphore means the transaction is shutting down, and a batch
                    // that ran anyway would be unbounded work during a shutdown. Every member
                    // is reported failed through the same shape a skipped node uses, so the
                    // summary counts them rather than the run looking like it lost them.
                    Err(e) => {
                        let said = Error::Transaction(format!("Semaphore failure: {}", e));
                        return batch
                            .iter()
                            .map(|(idx, action)| {
                                let (backend_name, package_name) = match action {
                                    GraphAction::Install(s) => (s.backend.clone(), s.name.clone()),
                                    GraphAction::Remove { backend, name } => {
                                        (backend.clone(), name.clone())
                                    }
                                };
                                TaskResult {
                                    node_index: *idx,
                                    result: Err(said.clone()),
                                    backend_name,
                                    package_name,
                                    retries: 0,
                                    duration: Duration::ZERO,
                                    bytes_downloaded: 0,
                                    start_time: chrono::Utc::now(),
                                    prior: Prior::Unknown,
                                    batch_size: 0,
                                }
                            })
                            .collect();
                    }
                };
                Self::execute_batch_with_retry(BatchArgs {
                    batch,
                    registry,
                    journal,
                    config,
                    app_config,
                    reaped,
                    hooks,
                    cancel_token,
                })
                .await
            });
        }
    }

    /// Record one joined batch, and answer whether the run must stop.
    fn absorb_results(&mut self, results: Vec<TaskResult>, run: &mut RunState) -> Option<Error> {
        // Every result is recorded before any failure is acted on. A batch that fails fails
        // every package in it, and reporting only the first would make the summary say one
        // package did not install when six did not.
        let mut first_failure: Option<Error> = None;
        let mut failed_now: Vec<(NodeIndex, String)> = Vec::new();
        // Whether every failure joined this round is one Shall classified as passing. Asked
        // here, where the error is still in scope, and not re-derived later from the message:
        // `retryability` is structured and a text match is a guess.
        let mut every_failure_passing = true;
        for task_data in results {
            if task_data.result.is_ok() {
                trace!(
                    "Node {}:{} succeeded.",
                    task_data.backend_name,
                    task_data.package_name
                );
                run.in_progress.remove(&task_data.node_index);
                self.completed_indices.insert(task_data.node_index);
                self.history
                    .push((task_data.node_index, task_data.prior.clone()));
                // Whatever was waiting only on this one is ready now.
                for dependent in self
                    .graph
                    .neighbors_directed(task_data.node_index, Direction::Outgoing)
                {
                    if let Some(n) = run.pending_deps.get_mut(&dependent) {
                        *n = n.saturating_sub(1);
                        if *n == 0 && !self.completed_indices.contains(&dependent) {
                            run.ready.push(dependent);
                        }
                    }
                }
                run.results.push(task_data);
                continue;
            }

            let error_msg = task_data
                .result
                .as_ref()
                .err()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "Execution Error".into());

            // `debug!`, not `error!`. This failure is returned below and printed once, as
            // itself, by `main`. Printing it here as well said the same thing twice and called
            // the package a "Node" — the DAG's word for it, which no user asked about. The
            // suggestions below are the part worth keeping.
            debug!(
                "Node {}:{} FAILED: {}",
                task_data.backend_name, task_data.package_name, error_msg
            );

            // Once per failure, not once per package in a batch: six packages sharing one
            // command share one reason, and printing the same paragraph six times is the noise
            // the diagnostics exist to cut through.
            if first_failure.is_none() {
                self.diagnostics
                    .print_suggestions(&error_msg, &task_data.backend_name);
                // Named here because this is the only place that still knows which node it
                // was. `install X` converges the whole configuration, so the line that failed
                // is often not the one anybody typed, and the error used to arrive as the
                // manager's own words about a command the user never asked for (`Q34`).
                let origin = match &self.graph[task_data.node_index] {
                    GraphAction::Install(s) => s.options.one("__source").map(str::to_string),
                    GraphAction::Remove { .. } => None,
                };
                first_failure = Some(task_data.result.clone().err().unwrap().about_declaration(
                    &format!("{}:{}", task_data.backend_name, task_data.package_name),
                    origin.as_deref(),
                ));
            }
            every_failure_passing &= matches!(
                task_data.result.as_ref().err().map(Error::retryability),
                Some(Retryability::Transient) | Some(Retryability::Exhausted)
            );
            failed_now.push((
                task_data.node_index,
                format!("{}:{}", task_data.backend_name, task_data.package_name),
            ));
            run.results.push(task_data);
        }

        if self.config.continue_past.carries_on(every_failure_passing) {
            // A failed node is terminal: it will not be retried and nothing waiting on it can
            // run, so both it and everything downstream come off the board here — otherwise the
            // loop below never reaches `total_nodes` and reports a cycle that does not exist.
            for (idx, named) in failed_now {
                run.in_progress.remove(&idx);
                self.completed_indices.insert(idx);
                for skipped in self.unreachable_from(idx) {
                    if self.completed_indices.insert(skipped) {
                        run.results.push(Self::skipped_result(
                            &self.graph[skipped],
                            skipped,
                            &named,
                        ));
                    }
                }
            }
            run.ready.sort();
            return None;
        }

        let final_err = first_failure?;
        run.ready.sort();
        Some(final_err)
    }

    /// Record the shape of a run that reached closure, and say when it ran more serially than
    /// its graph forced.
    fn report_scheduling(&mut self, run: &RunState) {
        // Only on the path where the loop ran to closure. A transaction that returned early went
        // idle fewer times than its graph has levels, so measuring it reports a shape no run had
        // — and `waves <= depth` being unfalsifiable there is exactly why the number is not
        // worth keeping.
        self.last_scheduling = Some(Scheduling {
            packages: run.remaining,
            depth: run.depth,
            waves: run.waves,
        });
        // The skip is announced and the pass is not. `scheduling_violation` returning `None`
        // means one of two quite different things — the plan was too small to have a scheduling
        // shape worth a rule, or the rule ran and the plan was within it — and this site used to
        // treat both as silence. The first is a measurement that did not happen and the second
        // is an answer, so the first is checked first and said out loud.
        if !crate::core::latency::scheduling_is_measurable(run.remaining, run.depth) {
            crate::core::latency::unmeasured(
                "the plan",
                &format!(
                    "{} package(s) with a longest dependency chain of {}, which is under the two \
                     packages the scheduling rule needs before waves above depth are reachable at \
                     all",
                    run.remaining, run.depth
                ),
            );
        } else if let Some(why) =
            crate::core::latency::scheduling_violation(run.remaining, run.depth, run.waves)
        {
            tracing::warn!(
                "this plan was executed more serially than it is shaped: {}. The seconds a \
                 package manager costs belong to the host; the order Shall asks in does not.",
                why
            );
        }
    }

    /// Every node that can only be reached through `failed` — the work a failure has just made
    /// impossible. Excludes `failed` itself, which the caller has already accounted for.
    fn unreachable_from(&self, failed: NodeIndex) -> Vec<NodeIndex> {
        let mut out = Vec::new();
        let mut stack: Vec<NodeIndex> = self
            .graph
            .neighbors_directed(failed, Direction::Outgoing)
            .collect();
        let mut seen: HashSet<NodeIndex> = HashSet::new();
        while let Some(idx) = stack.pop() {
            if !seen.insert(idx) {
                continue;
            }
            out.push(idx);
            stack.extend(self.graph.neighbors_directed(idx, Direction::Outgoing));
        }
        out.sort();
        out
    }

    /// A node nobody attempted, reported as itself. Not a success and not a failure of its own:
    /// the reason names the node that stopped it, because "jq failed" about a package Shall
    /// never ran a command for is the attribution problem this engine is meant to be free of.
    fn skipped_result(action: &GraphAction, idx: NodeIndex, blocked_by: &str) -> TaskResult {
        let (backend_name, package_name) = match action {
            GraphAction::Install(s) => (s.backend.clone(), s.name.clone()),
            GraphAction::Remove { backend, name } => (backend.clone(), name.clone()),
        };
        TaskResult {
            node_index: idx,
            result: Err(Error::Transaction(format!(
                "not attempted: it needs `{}`, which could not be completed",
                blocked_by
            ))),
            backend_name,
            package_name,
            retries: 0,
            duration: Duration::ZERO,
            bytes_downloaded: 0,
            start_time: chrono::Utc::now(),
            prior: Prior::Unknown,
            batch_size: 0,
        }
    }

    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    /// The most packages Shall will put on one manager command line.
    ///
    /// A bound on argv, not on ambition: `cmd.exe` caps a command line at 8191 characters and
    /// every manager has some limit. A hundred names is far below any of them and far above
    /// the point where batching has taken the win — the cost this removes is per *invocation*,
    /// so the second hundred saves almost nothing the first did not.
    pub const MAX_BATCH: usize = 100;
    /// …and a byte bound, because package names are not all short. `github:owner/repo@…`
    /// spends far more per name than `jq` does.
    const MAX_BATCH_BYTES: usize = 6000;

    /// Split a ready set into the commands it becomes.
    ///
    /// Everything in one batch shares a manager and a kind of change, and no two of them have
    /// an edge between them — they are ready at the same moment, which is what "ready" means.
    /// Batches come out in node order so a plan runs the same way twice.
    ///
    /// **Every edge in this graph is an `@requires` somebody wrote** (`Y9`). The planner used
    /// to add one per native dependency it discovered, which split this batch for a
    /// relationship the manager was going to resolve by itself anyway.
    ///
    /// `max_batch` is a cap the caller chooses rather than [`Self::MAX_BATCH`] outright,
    /// because **batching and `--keep-going` are in direct contradiction and batching used to
    /// win silently.** One name no repository carries fails the whole `apt install`, so a run
    /// with `--keep-going` — whose help promises to "finish the packages that still can" —
    /// installed none of the good packages sharing that command line (B1). A flag whose entire
    /// purpose is taking what it can get does not want them on one command line; it wants each
    /// package to succeed or fail on its own. That costs invocations, and invocations are what
    /// this flag is explicitly willing to spend.
    pub fn batches(
        graph: &StableDiGraph<GraphAction, ()>,
        mut ready: Vec<NodeIndex>,
        max_batch: usize,
    ) -> Vec<Batch> {
        ready.sort();
        /// One manager, one kind of change, and the nodes gathered for it so far.
        struct Group {
            backend: String,
            is_install: bool,
            members: Batch,
        }
        let mut groups: Vec<Group> = Vec::new();
        for idx in ready {
            let action = graph[idx].clone();
            let (backend, is_install) = match &action {
                GraphAction::Install(s) => (s.backend.clone(), true),
                GraphAction::Remove { backend, .. } => (backend.clone(), false),
            };
            match groups
                .iter_mut()
                .find(|g| g.backend == backend && g.is_install == is_install)
            {
                Some(g) => g.members.push((idx, action)),
                None => groups.push(Group {
                    backend,
                    is_install,
                    members: vec![(idx, action)],
                }),
            }
        }

        let mut out = Vec::new();
        for Group { members, .. } in groups {
            let mut current: Batch = Vec::new();
            let mut bytes = 0usize;
            for (idx, action) in members {
                let cost = match &action {
                    GraphAction::Install(s) => s.name.len() + 1,
                    GraphAction::Remove { name, .. } => name.len() + 1,
                };
                if !current.is_empty()
                    && (current.len() >= max_batch.max(1) || bytes + cost > Self::MAX_BATCH_BYTES)
                {
                    out.push(std::mem::take(&mut current));
                    bytes = 0;
                }
                bytes += cost;
                current.push((idx, action));
            }
            if !current.is_empty() {
                out.push(current);
            }
        }
        out
    }

    /// What the package looks like right now, before this node touches it.
    async fn prior_state(backend_cap: &Arc<crate::core::BackendCapabilities>, name: &str) -> Prior {
        let Some(q) = backend_cap.as_queryable() else {
            return Prior::Unknown;
        };
        match q.info(name).await {
            Ok(Some(pkg)) => Prior::Present(pkg.version),
            Ok(None) => Prior::Absent,
            // A query that failed is not a package that is absent. Reading it as one is how
            // a rollback ends up removing software this run never installed.
            Err(_) => Prior::Unknown,
        }
    }

    /// Stop every outstanding worker, and wait until the managers it was running are done
    /// dying.
    ///
    /// `abort_all` only *requests* cancellation; the dropped worker's guard sends SIGTERM and,
    /// being a `Drop`, cannot stay to watch. A manager takes up to the termination grace after
    /// that signal to release its package database, and every command the rollback is about to
    /// issue competes for exactly those locks — a rollback started here without this wait fails
    /// on wounds this engine itself inflicted. Windows has no graceful stop to wait out, but a
    /// killed process still needs a moment before its locks are gone.
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    pub async fn quiesce_workers(worker_pool: &mut JoinSet<Vec<TaskResult>>) {
        if worker_pool.is_empty() {
            return;
        }
        worker_pool.abort_all();
        while worker_pool.join_next().await.is_some() {}
        #[cfg(unix)]
        let settle = crate::core::executor::RawExecutor::TERMINATION_GRACE;
        #[cfg(not(unix))]
        let settle = std::time::Duration::from_secs(2);
        tokio::time::sleep(settle).await;
    }

    /// Removals this transaction carried out that **stayed removed**, as `(backend, name)`
    /// pairs.
    ///
    /// Read after a failed run: `history` records every action that completed, but not all
    /// of them still hold. Rollback reinstates a completed removal unless the plan itself
    /// intended the package gone (`U41`), and one whose target was never there changed
    /// nothing — so only what survives both tests may be reported as gone. This is what lets
    /// a caller summarise a run that died part-way instead of telling the user
    /// all-or-nothing.
    pub fn executed_removals(&self) -> Vec<(String, String)> {
        self.history
            .iter()
            .filter_map(|(idx, prior)| {
                let GraphAction::Remove { name, backend } = &self.graph[*idx] else {
                    return None;
                };
                let stays_gone = !matches!(prior, Prior::Absent)
                    && self.plan_intends_present(backend, name) == Some(false);
                stays_gone.then(|| (backend.clone(), name.clone()))
            })
            .collect()
    }
}

/// What to do about a failed attempt whose manager said its lock was taken.
#[derive(Debug)]
pub enum LockWait {
    /// A live holder, named. Wait for it.
    Wait(String),
    /// Waiting would never end. Fail now, with the sentence that says why.
    Hopeless(Error),
    /// Not a lock failure, or the lock is free again. The ordinary backoff.
    Backoff,
}

/// How much of `manager_lock_wait_secs` a batch has left to spend waiting for another manager.
///
/// **One budget across the whole retry loop, not one per attempt** — a queue of holders taking
/// the lock in turn is a real machine state, and three full waits in a row would be three times
/// the bound the setting promises. Written as a type for the reason [`backoff_for`] was written
/// as a function: the running total was a `Duration` accumulated at one site and subtracted at
/// another, and neither arithmetic could be reached without a second package manager to hold a
/// real lock. Here they are two named operations with a test each.
#[derive(Debug, Clone, Copy)]
pub struct LockBudget {
    total: Duration,
    spent: Duration,
}

impl LockBudget {
    pub fn of(total: Duration) -> Self {
        Self {
            total,
            spent: Duration::ZERO,
        }
    }

    /// The bound the setting promised, before any of it was spent.
    ///
    /// Read by the verdict so "the user set 0" and "the user set 300 and Shall spent it" can
    /// be told apart — the old message said `manager_lock_wait_secs` is 0 to both, which to
    /// someone who set 300 is a sentence about a decision they never made.
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn total(&self) -> Duration {
        self.total
    }

    /// What is left to wait with. Saturating, because a wait that overran its share leaves
    /// nothing rather than a negative bound — and zero is the value `lock_wait_verdict` reads as
    /// "do not wait", which is the right answer once the budget is gone.
    pub fn remaining(&self) -> Duration {
        self.total.saturating_sub(self.spent)
    }

    /// Charge a wait that has already happened.
    pub fn spend(&mut self, waited: Duration) {
        self.spent += waited;
    }
}

/// Which of the three a failure is.
///
/// The verdict is taken from the machine and not from the message: the manager only says *"could
/// not get lock"*, and whether that is a queue to join or a corpse to clear is the difference
/// between waiting five minutes and waiting forever. `/proc` knows; the string does not.
/// `look` is how the machine is asked, so the three verdicts can be exercised without a second
/// package manager to kill. It is called only *after* the manager's own words have matched, which
/// is what keeps a successful install from ever reading `/proc`.
pub fn lock_wait_verdict(
    last_error: &Option<Error>,
    backend: &str,
    remaining: Duration,
    budget_total: Duration,
    look: &dyn Fn(&str) -> crate::app::stale_lock::Held,
) -> LockWait {
    let Some(err) = last_error else {
        return LockWait::Backoff;
    };
    if !crate::app::stale_lock::says_the_lock_is_taken(backend, &err.to_string()) {
        return LockWait::Backoff;
    }
    match look(backend) {
        crate::app::stale_lock::Held::Live(who) if !remaining.is_zero() => LockWait::Wait(who),
        // Opted out of waiting — either the setting is 0 or Shall spent every second of it.
        // The message says which, because "is 0" to someone who set 300 is a sentence about a
        // decision they never made. Both endings are still the true one rather than the old
        // "a further retry will not help": a further retry is exactly what would help, once
        // the holder is done.
        crate::app::stale_lock::Held::Live(who) => {
            let why = if budget_total.is_zero() {
                "`manager_lock_wait_secs` is 0, so Shall did not wait for it. Raise that \
                 setting or run this again once the other manager has finished."
                    .to_string()
            } else {
                format!(
                    "Shall waited its full {}s of `manager_lock_wait_secs` and the lock never \
                     came free. Run this again once the other manager has finished, or raise \
                     the setting.",
                    budget_total.as_secs()
                )
            };
            LockWait::Hopeless(Error::CommandFailed {
                message: format!(
                    "`{backend}` cannot run: {who} holds the manager's lock, and {why}"
                ),
                retry: Retryability::Exhausted,
                absent_name: false,
            })
        }
        crate::app::stale_lock::Held::Stale(path) => LockWait::Hopeless(Error::CommandFailed {
            message: format!(
                "`{backend}` cannot run: {} is on disk and nothing holds it — a run of this \
                 manager was killed and left its lock behind. Waiting will not clear it. \
                 `shall heal` removes exactly this, after proving again that no manager is \
                 running.",
                path.display()
            ),
            retry: Retryability::Exhausted,
            absent_name: false,
        }),
        crate::app::stale_lock::Held::Free => LockWait::Backoff,
    }
}

/// Wait for whoever holds the manager's lock, and say so while waiting.
///
/// `None` means the lock came free and the caller should try again. `Some(err)` is the wait
/// ending without that happening, and it says which of the two ways it ended.
///
/// **A wait with no reason given is indistinguishable from a hang**, and a hang is what people
/// kill — which is how a machine ends up with the interrupted transaction this whole module is
/// about. It announces once, up front, the way the data-directory lock does.
pub async fn wait_for_manager_lock(
    backend: &str,
    who: &str,
    wait: Duration,
    cancel_token: &CancellationToken,
) -> std::result::Result<Duration, Error> {
    eprintln!(
        "shall: waiting for {who} to finish — it holds the lock `{backend}` needs \
         (up to {}s; `manager_lock_wait_secs` sets that)",
        wait.as_secs()
    );
    // The polling loop is `stale_lock`'s, because `heal` waits on the same question and the two
    // must not drift. What is decided here is only what to say about each ending.
    match crate::app::stale_lock::wait_until_not_held(backend, wait, &|| {
        cancel_token.is_cancelled()
    })
    .await
    {
        // It let go. Whether it finished or died, the next attempt is the thing that finds out,
        // and a stale lock left by a holder that died mid-wait is reported by the next pass
        // through the verdict rather than guessed at here.
        crate::app::stale_lock::Waited::Freed(spent) => {
            info!(
                "the lock `{}` needs came free after {}s",
                backend,
                spent.as_secs()
            );
            return Ok(spent);
        }
        crate::app::stale_lock::Waited::Cancelled => return Err(Error::Cancelled),
        crate::app::stale_lock::Waited::StillHeld => {}
    }
    Err(Error::CommandFailed {
        message: format!(
            "`{backend}` cannot run: {who} has held the manager's lock for {}s, which is all \
             `manager_lock_wait_secs` allows. It is still running, so nothing here is broken and \
             nothing needs clearing — run this again when it has finished, or raise that setting.",
            wait.as_secs()
        ),
        retry: Retryability::Exhausted,
        absent_name: false,
    })
}

/// The sentence a batch member's WAL entry carries when it did not succeed.
///
/// **Named so it can be asserted.** It was a match guard three lines into a journal loop, and
/// the nightly mutation shard replaced that guard with `true` and with `false` - both survived
/// the whole suite, because nothing anywhere read which of the two sentences got written. They
/// are different facts to whoever reads the WAL afterwards: one says another operation in this
/// run failed, the other says this one did.
pub fn wal_failure_reason(cancelled: bool, error: &Error) -> String {
    if cancelled {
        "cancelled before this batch ran: another operation in the same run failed".to_string()
    } else {
        format!("{}", error)
    }
}

/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so a private item is invisible there.
/// How many of `attempt` tries were retries: all but the first.
///
/// `attempt` counts from 1, so a batch that succeeded first time reports nought. Written out
/// because three `TaskResult` constructions computed it inline, and a copy of an expression is
/// a copy of its blind spot — the mutation sweep found the same subtraction three times over
/// and nothing in the suite could tell any of them from `attempt + 1`.
pub fn retries_behind(attempt: u32) -> u32 {
    attempt - 1
}

/// The wait before retry number `attempt`, doubling each time and capped at `max`.
///
/// Only ever called from inside `attempt > 1`, so the first retry waits `initial` exactly and
/// the exponent cannot go negative. Written out for the same reason as [`retries_behind`], with
/// more to answer for: a shift inside a multiplication inside a `min` inside a match arm is
/// three separate numbers, and all three survived — `<<` read as `>>`, and `attempt - 2` read
/// as both `attempt + 2` and `attempt / 2`, without failing anything.
pub fn backoff_for(attempt: u32, initial: Duration, max: Duration) -> Duration {
    std::cmp::min(initial * (1 << (attempt - 2)), max)
}

/// A failure that survived its own retries is not transient, whatever the string said.
///
/// `Retryability::Transient` is a claim: *a second attempt could differ*. The container harness
/// proves that claim the only way it can be proved — it retries once and calls a repeat a
/// defect. The product asserted it from a substring and never checked, so `luarocks install
/// luafilesystem` on a machine whose `wget` is a scoop shim matched `"failed downloading"`,
/// was called transient, and told the user `sync` would try it again. It fails identically
/// forever; `exit_policy::luarocks`'s own doc comment names that exact cause and classifies it
/// as the network anyway.
///
/// The evidence was already being collected and thrown away. This loop retries a transient
/// failure with backoff, so by the time it gives up it **has** re-run the command and seen the
/// same answer. That is the experiment; this records its result. `Unknown` rather than
/// `Permanent`, because "we tried and it did not differ" is not "this can never work" — the
/// wget on the PATH could be fixed tomorrow. Withdrawing a declaration is not this function's
/// to trigger either way: that reads `Error::says_a_name_is_absent`, and no amount of repeating
/// turns "the download failed" into "the rock does not exist".
pub fn falsify_transience(err: Error, attempts: u32) -> Error {
    if attempts < 2 {
        return err; // never retried, so nothing was tested
    }
    match err {
        Error::CommandFailed {
            message,
            retry: Retryability::Transient,
            absent_name,
        } => Error::CommandFailed {
            message: format!(
                "{} (tried {} times; the failure did not change, so a further retry will not \
                 help — this is not the transient failure its output looks like)",
                message, attempts
            ),
            retry: Retryability::Exhausted,
            // Carried, not recomputed. Nothing here re-reads the manager's output, so
            // dropping the flag would turn "the name is not there" into "something failed
            // repeatedly" purely by passing through the retry loop.
            absent_name,
        },
        other => other,
    }
}
