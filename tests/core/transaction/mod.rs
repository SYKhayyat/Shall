use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use shall::backends::BackendRegistry;
pub use shall::core::batch::BatchRecovery;
use shall::core::journal::JournalAction;
pub use shall::core::transaction::*;
use shall::core::{Error, Journal, PackageSpec, Result, Retryability};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
async fn tx_with(
    graph: StableDiGraph<GraphAction, ()>,
    history: Vec<(NodeIndex, Prior)>,
    declared: Option<&[&str]>,
) -> Transaction {
    let config = shall::config::Config::default();
    let journal_dir = tempfile::tempdir().unwrap();
    let journal = Journal::at(journal_dir.path().join("journal.jsonl")).unwrap();
    let diagnostics = shall::app::diagnostics::FailureDiagnosticEngine::init(&config).await;
    let mut tx = Transaction::with_config(
        graph,
        Arc::new(BackendRegistry::new()),
        Arc::new(Mutex::new(journal)),
        Arc::new(diagnostics),
        Arc::new(config),
        TransactionConfig::default(),
    );
    tx.history = history;
    tx.declared =
        declared.map(|d| Arc::new(d.iter().map(|s| s.to_string()).collect::<HashSet<String>>()));
    tx
}
fn remove_node(graph: &mut StableDiGraph<GraphAction, ()>, backend: &str, name: &str) -> NodeIndex {
    graph.add_node(GraphAction::Remove {
        backend: backend.to_string(),
        name: name.to_string(),
    })
}
pub use shall::app::stale_lock::Held;
fn lock_failure(msg: &str) -> Option<Error> {
    Some(Error::CommandFailed {
        message: msg.to_string(),
        retry: Retryability::Transient,
        absent_name: false,
    })
}
const PACMAN_SAID: &str = "`pacman` failed (exit 1): error: failed to init transaction \
                           (unable to lock database)";
fn transient(msg: &str) -> Error {
    Error::CommandFailed {
        message: msg.to_string(),
        retry: Retryability::Transient,
        absent_name: false,
    }
}
pub use shall::core::journal::ActionStatus;
pub use shall::core::manager::{BackendCapabilities, BackendCore, HealthReport, HealthStatus};
pub use shall::core::{Installable, Package, Queryable};
use std::sync::atomic::{AtomicUsize, Ordering};
struct Counting {
    name: String,
    calls: AtomicUsize,
    widest: AtomicUsize,
    listings: shall::core::installed::InstalledListings,
    /// How long this backend takes to answer. Zero for every test but the ones that need
    /// a batch to still be in flight when the run stops.
    stall: Duration,
    /// Whether this backend refuses everything, so a test can make one manager fail while
    /// another is mid-command.
    fails: bool,
}
#[async_trait::async_trait]
impl BackendCore for Counting {
    fn name(&self) -> &str {
        &self.name
    }
    fn is_available(&self) -> bool {
        true
    }
    fn probes(&self) -> Vec<String> {
        Vec::new()
    }
    fn needs_root(&self) -> bool {
        false
    }
    async fn check_health(&self) -> Result<HealthReport> {
        Ok(HealthReport {
            status: HealthStatus::Ok,
            message: None,
        })
    }
}
#[async_trait::async_trait]
impl Installable for Counting {
    async fn install(&self, specs: &[PackageSpec], _sudo: bool) -> Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.widest.fetch_max(specs.len(), Ordering::SeqCst);
        tokio::time::sleep(self.stall).await;
        self.answer()
    }
    async fn remove(
        &self,
        names: &[String],
        _sudo: bool,
        _reaped: shall::app::sync::guard::Reaped,
    ) -> Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.widest.fetch_max(names.len(), Ordering::SeqCst);
        tokio::time::sleep(self.stall).await;
        self.answer()
    }
}
impl Counting {
    fn answer(&self) -> Result<()> {
        match self.fails {
            true => Err(Error::Transaction(format!("`{}` refuses", self.name))),
            false => Ok(()),
        }
    }
}
#[async_trait::async_trait]
impl Queryable for Counting {
    fn installed_cache(&self) -> (&shall::core::installed::InstalledListings, &str) {
        (&self.listings, &self.name)
    }
    async fn fetch_installed(&self) -> Result<Vec<Package>> {
        Ok(Vec::new())
    }
    async fn list_manual(&self) -> Result<Vec<Package>> {
        Ok(Vec::new())
    }
    async fn info(&self, _name: &str) -> Result<Option<Package>> {
        Ok(None)
    }
}
fn spec(backend: &str, name: &str) -> PackageSpec {
    PackageSpec {
        name: name.to_string(),
        backend: backend.to_string(),
        options: Default::default(),
        requires: Vec::new(),
        present: true,
    }
}
struct Harness {
    tx: Transaction,
    counters: Vec<Arc<Counting>>,
    /// The same log the transaction writes to, so a test can ask what it was left holding.
    journal: Arc<Mutex<Journal>>,
    _tmp: tempfile::TempDir,
}
async fn harness(graph: StableDiGraph<GraphAction, ()>, backends: &[&str]) -> Harness {
    harness_with(
        graph,
        backends,
        Duration::ZERO,
        Duration::from_secs(3600),
        &[],
    )
    .await
}
async fn harness_with(
    graph: StableDiGraph<GraphAction, ()>,
    backends: &[&str],
    stall: Duration,
    total_timeout: Duration,
    failing: &[&str],
) -> Harness {
    let tmp = tempfile::tempdir().unwrap();
    let mut registry = BackendRegistry::new();
    let mut counters = Vec::new();
    for b in backends {
        let fails = failing.contains(b);
        let counting = Arc::new(Counting {
            name: b.to_string(),
            calls: AtomicUsize::new(0),
            widest: AtomicUsize::new(0),
            listings: shall::core::installed::InstalledListings::new(),
            // A backend that is meant to fail does it at once; the stall exists to keep the
            // *other* one inside its command while that happens.
            stall: if fails { Duration::ZERO } else { stall },
            fails,
        });
        counters.push(counting.clone());
        registry.register(Arc::new(
            BackendCapabilities::builder(counting.clone())
                .with_installable(counting.clone())
                .with_queryable(counting)
                .build(),
        ));
    }
    let config = shall::config::Config::default();
    let journal = Arc::new(Mutex::new(
        Journal::at(tmp.path().join("journal.jsonl")).unwrap(),
    ));
    let diagnostics = shall::app::diagnostics::FailureDiagnosticEngine::init(&config).await;
    let mut tx_config = TransactionConfig::patient();
    // Rollback off: it would ask each backend what was there before, which is not what
    // these tests are measuring.
    tx_config.auto_rollback = false;
    tx_config.total_timeout = total_timeout;
    let tx = Transaction::with_config(
        graph,
        Arc::new(registry),
        journal.clone(),
        Arc::new(diagnostics),
        Arc::new(config),
        tx_config,
    )
    // These tests hand the executor a graph directly, which is precisely the case the
    // `reaped` refusal exists to catch in production â€” a plan that reached the engine
    // without passing the guard. What they are measuring is how the executor *batches*,
    // and threading a real `Config` and `BackendRegistry` through a guard to measure that
    // would prove nothing about either.
    .guarded_by(crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Sync).await);
    Harness {
        tx,
        counters,
        journal,
        _tmp: tmp,
    }
}

mod a_backend_with_no_manager_lock_backs_off_as_before;
mod a_cancelled_batch_and_a_failed_one_leave_different_sentences;
mod a_cancelled_run_stops_waiting_immediately;
mod a_command_line_is_bounded;
mod a_failure_that_is_not_about_a_lock_never_asks_the_machine;
mod a_failure_that_was_never_retried_keeps_its_classification;
mod a_live_holder_is_waited_for;
mod a_lock_that_came_free_goes_back_to_the_backoff;
mod a_permanent_failure_is_not_touched_by_the_retry_count;
mod a_removal_of_something_absent_is_not_counted_as_gone;
mod a_removal_that_was_not_intended_is_treated_as_reinstated;
mod a_removal_the_plan_intended_and_rollback_left_gone_is_reported;
mod a_requires_edge_still_orders_the_two_sides;
mod a_run_stopped_by_one_managers_failure_closes_its_entries_as_abandoned;
mod a_run_that_times_out_closes_its_entries_as_abandoned;
mod a_spent_budget_is_zero_rather_than_a_negative_one;
mod a_stale_lock_fails_at_once_and_names_heal;
mod a_transient_failure_that_repeated_stops_calling_itself_transient;
mod an_earlier_runs_open_entry_survives_this_runs_failure;
mod an_exhausted_budget_does_not_claim_the_setting_was_zero;
mod an_install_and_a_removal_never_share_a_command;
mod an_install_in_the_history_is_not_a_removal;
mod an_unknown_failure_is_left_alone;
mod every_setting_from_config_claims_to_read_reaches_the_transaction;
mod no_failure_yet_is_not_a_lock_failure;
mod one_rollback_rule_answers_both_directions;
mod only_a_reconciling_run_may_leave_a_removal_in_place;
mod opting_out_of_the_wait_still_says_something_true;
mod retries_are_every_attempt_but_the_first;
mod six_independent_installs_are_one_command;
mod the_backoff_doubles_from_the_first_retry_and_stops_at_the_cap;
mod the_byte_bound_splits_exactly_where_the_cost_says;
mod the_engine_reports_the_shape_it_actually_ran_in;
mod the_lock_budget_reports_the_bound_it_was_given;
mod the_second_attempt_is_already_a_repeat;
mod the_transaction_leaves_no_worker_running_behind_it;
mod the_wait_budget_is_spent_across_attempts_and_not_per_attempt;
mod two_managers_are_two_commands_and_not_one;
