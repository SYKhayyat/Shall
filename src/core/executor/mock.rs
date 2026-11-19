//! The test double every mock-driven suite runs against, with its two ledgers.

use super::*;

/// The test double every mock-driven suite runs against.
///
/// **An unregistered command used to return `Ok(DryRunOutput::new())` — empty, success.** That
/// is the same "silence means fine" default the parser layer had, one storey up, and it does the
/// same damage: a test registers a stub, the product emits a slightly different argv, the stub is
/// never matched, the call falls through to the default, and the test passes having asserted
/// nothing.
///
/// It was not hypothetical. The since-deleted `e2e_tests.rs` registered `"brew install {name}"` while the
/// product emits `"brew install -- neovim"` (`argv.rs`); all five of that file's
/// registrations were dead strings and every test passed on the default. Sixteen lines away in
/// the same suite, `a_machine_converges_tests.rs` registered the `--` form. **Two tests
/// disagreed about the product's own argv and both were green.**
///
/// So the mock now keeps two ledgers, and the second is the one that catches that bug:
///
/// - **`unstubbed`** — commands that ran with no registration. Recorded rather than refused,
///   because a great many tests legitimately do not care what a command printed; they assert the
///   argv, or the state afterwards. **It was read by nothing at all**, so it was a record rather
///   than an instrument; it has a read and a per-test opt-in now. `V.215`.
/// - **`unmatched registrations`** — a stub that was set and never used. There is no innocent
///   reading of that: the test author wrote down what they expected the product to run, and the
///   product ran something else. It fails the test at drop.
///
/// The two are independent by construction, and the allowance for one is checked before the
/// other is consulted, so neither opt-in can switch off the other's check.
pub struct MockExecutor {
    pub responses: DashMap<String, Result<StdOutput>>,
    pub command_existence: DashMap<String, bool>,
    pub call_log: Arc<Mutex<Vec<String>>>,
    /// The environment the last call carried. The env map is where the pager suppression and
    /// the recursion guard live, and neither is visible in the argv the call log records.
    pub last_env: Arc<Mutex<HashMap<String, String>>>,
    /// Which registered patterns were actually matched by a call.
    matched: DashMap<String, ()>,
    /// Commands that ran with nothing registered for them, deduplicated.
    unstubbed: DashMap<String, ()>,
    /// Registrations whose whole purpose is to stay unmatched — see
    /// [`MockExecutor::set_response_that_must_not_be_used`].
    forbidden: DashMap<String, ()>,
    /// Standing facts about the machine — see [`MockExecutor::set_machine_response`].
    machine: DashMap<String, ()>,
    /// The answer for every command with no exact registration — see
    /// [`MockExecutor::answer_everything_with`].
    default_response: std::sync::Mutex<Option<Result<StdOutput>>>,
    /// How long a command takes, for tests about concurrency rather than about output.
    delays: DashMap<String, std::time::Duration>,
    /// Set by a test that means it — see [`MockExecutor::allow_unmatched_registrations`].
    allow_unmatched: std::sync::atomic::AtomicBool,
    /// Set by a test that means it — see [`MockExecutor::deny_unstubbed`].
    deny_unstubbed: std::sync::atomic::AtomicBool,
    /// Whether this mock has already reported its fall-throughs in audit mode.
    ///
    /// Audit mode reports once per mock rather than once per call, so a run's output stays readable
    /// and one test cannot flood the log.
    audited: std::sync::atomic::AtomicBool,
    vfs: Arc<DashMap<PathBuf, String>>,
}

impl MockExecutor {
    pub fn new(vfs: Arc<DashMap<PathBuf, String>>) -> Self {
        Self {
            responses: DashMap::new(),
            command_existence: DashMap::new(),
            call_log: Arc::new(Mutex::new(Vec::new())),
            last_env: Arc::new(Mutex::new(HashMap::new())),
            matched: DashMap::new(),
            unstubbed: DashMap::new(),
            forbidden: DashMap::new(),
            machine: DashMap::new(),
            default_response: std::sync::Mutex::new(None),
            delays: DashMap::new(),
            allow_unmatched: std::sync::atomic::AtomicBool::new(false),
            deny_unstubbed: std::sync::atomic::AtomicBool::new(false),
            audited: std::sync::atomic::AtomicBool::new(false),
            vfs,
        }
    }

    pub fn set_response(&self, cmd_pattern: &str, response: Result<StdOutput>) {
        self.responses.insert(cmd_pattern.to_string(), response);
    }

    /// Make a command take time, so a test can observe two calls overlapping — or failing to.
    pub fn set_delay(&self, cmd_pattern: &str, delay: std::time::Duration) {
        self.delays.insert(cmd_pattern.to_string(), delay);
    }

    /// Register a convincing answer for a command the product must **not** run.
    ///
    /// **The foil, made into an assertion.** Five tests in this repo registered the *wrong*
    /// listing beside the right one — `dpkg-query -W` next to `apt-mark showmanual`, the
    /// essential query next to the manual one — to say *"and if adopt asks this instead, it will
    /// get a different set of packages and the assertion below will notice."* Every one of those
    /// stubs was dead, and its deadness was the whole point and was checked by nothing. Wire the
    /// product to the wrong listing and the test would have gone green, because the names it
    /// asserted happened to overlap.
    ///
    /// Registered this way, the stub answers if it is ever reached — so a product that asks the
    /// wrong question gets a wrong-shaped answer rather than empty success — and going unreached
    /// is asserted rather than assumed.
    pub fn set_response_that_must_not_be_used(
        &self,
        cmd_pattern: &str,
        response: Result<StdOutput>,
    ) {
        self.responses.insert(cmd_pattern.to_string(), response);
        self.forbidden.insert(cmd_pattern.to_string(), ());
    }

    /// Answer every command with no exact registration, and record none of them as unanswered.
    ///
    /// For a test that drives a whole surface to read the **argv** and asserts nothing about
    /// output: `argv_drift_tests` walks every backend's subcommand chain and asks whether those
    /// subcommands still exist upstream, so what each call returned is not a fact it has an
    /// opinion about. The ledger stays honest because this is a *written* answer rather than
    /// silence — the test says "every question here is answered with success", which is a claim
    /// that can be wrong, and the unused-stub check still applies to everything registered by
    /// name.
    ///
    /// Not a fixture default: a `TestKernel` test that reaches an unexpected command is a test
    /// whose machine is not what it thinks, and that stays reportable.
    pub fn answer_everything_with(&self, response: Result<StdOutput>) {
        *self
            .default_response
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(response);
    }

    /// Register a **standing fact about the machine** — what a manager holds — rather than this
    /// test's expectation about one call.
    ///
    /// The unused-stub check exists because a stub nobody matched is a claim about argv that the
    /// product disagreed with. A manager's whole listing is not that claim: it is the machine, and
    /// which part of it a given run consults is a property of the config, not of the product's
    /// dialect. So the answer is registered normally and **this key alone** is excused from the
    /// unused check — the mock's other stubs stay held to account, which is the difference between
    /// this and [`MockExecutor::allow_unmatched_registrations`].
    ///
    /// It is not for a per-package answer (`brew info -- pkg`), an install or a removal. Those are
    /// the test's own subject and are written down, and checked, where the test asserts them.
    pub fn set_machine_response(&self, cmd_pattern: &str, response: Result<StdOutput>) {
        self.responses.insert(cmd_pattern.to_string(), response);
        self.machine.insert(cmd_pattern.to_string(), ());
    }

    /// Every pattern this mock holds an answer for, sorted.
    ///
    /// The unused-stub check reads [`unmatched_registrations`](Self::unmatched_registrations),
    /// which deliberately hides the machine facts and the forbidden list. A test asking "is the
    /// answer I meant to register actually in here" wants the whole table, and asking it of the
    /// filtered list is how a correct registration reads as absent.
    pub fn registered_registrations(&self) -> Vec<String> {
        let mut out: Vec<String> = self.responses.iter().map(|e| e.key().clone()).collect();
        out.sort();
        out
    }

    /// Registered patterns no call ever matched.
    pub fn unmatched_registrations(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .responses
            .iter()
            .map(|e| e.key().clone())
            .filter(|k| {
                !self.matched.contains_key(k)
                    && !self.forbidden.contains_key(k)
                    && !self.machine.contains_key(k)
            })
            .collect();
        out.sort();
        out
    }

    /// For a test that registers a stub for a path it knows may not be taken — a fallback the
    /// product only reaches on another platform, or the second half of an either/or.
    ///
    /// Deliberately a per-mock opt-in with no default: the point of the check is that somebody
    /// has to look at the dead string and say it is meant to be dead.
    pub fn allow_unmatched_registrations(&self) {
        self.allow_unmatched
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Every command that ran with nothing registered for it, deduplicated and sorted.
    ///
    /// **Append-only**: registering a response afterwards does not remove the record.
    pub fn unstubbed_registrations(&self) -> Vec<String> {
        let mut out: Vec<String> = self.unstubbed.iter().map(|e| e.key().clone()).collect();
        out.sort();
        out
    }

    /// Fail if any command ran with nothing registered for it.
    ///
    /// Per-mock, no default: 172 of the suite's tests leave a command unanswered, and 106 of
    /// those are existence probes whose `true` default is deliberate, so a global check would
    /// redden a quarter of the suite for having no opinion. Independent of
    /// [`MockExecutor::allow_unmatched_registrations`], and cannot be switched off by it.
    pub fn deny_unstubbed(&self) {
        self.deny_unstubbed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Fail if any registered stub went unused.
    ///
    /// Called from `Drop` so that a test does not have to remember; exposed because a test that
    /// wants the failure at a particular point reads better than one that gets it at the end of
    /// the scope.
    pub fn assert_every_registration_was_used(&self) {
        // The forbidden half first: a stub registered as one the product must not run, and run
        // anyway, is a louder failure than an unused one and must not be masked by it.
        let ran_anyway: Vec<String> = self
            .forbidden
            .iter()
            .map(|e| e.key().clone())
            .filter(|k| self.matched.contains_key(k))
            .collect();
        assert!(
            ran_anyway.is_empty(),
            "the product ran {} command(s) this test says it must not:

  {}

             Registered with `set_response_that_must_not_be_used`, which means the test's \
             claim is that this question is never asked.",
            ran_anyway.len(),
            ran_anyway.join(
                "
  "
            )
        );

        // Checked BEFORE the allowance below, and the order is load-bearing: one flag must not
        // be able to switch off the other half's check. `an_unanswered_command_is_a_defect_tests`
        // holds the case that would fail if these two lines were swapped.
        if self
            .deny_unstubbed
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let unstubbed = self.unstubbed_registrations();
            if !unstubbed.is_empty() {
                let ran = self
                    .call_log
                    .try_lock()
                    .map(|l| l.clone())
                    .unwrap_or_default();
                panic!(
                    "{} command(s) ran with no mock response registered for them:\n\n  {}\n\n\
                     What actually ran:\n\n  {}\n\n\
                     Each fell through to the empty-success default, so every assertion this test \
                     makes about what Shall PRINTED proved nothing. Asserting the argv, or the \
                     state afterwards, is a fine reason to leave one unstubbed — but this test \
                     called `deny_unstubbed()`. Register the answer, or drop the call. `V.215`.",
                    unstubbed.len(),
                    unstubbed.join("\n  "),
                    if ran.is_empty() {
                        "(nothing)".to_string()
                    } else {
                        ran.join("\n  ")
                    }
                );
            }
        }

        if self
            .allow_unmatched
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return;
        }
        let unused = self.unmatched_registrations();
        if unused.is_empty() {
            return;
        }
        let ran = self
            .call_log
            .try_lock()
            .map(|l| l.clone())
            .unwrap_or_default();
        panic!(
            "{} registered mock response(s) were never matched by any call:\n\n  {}\n\n\
             What actually ran:\n\n  {}\n\n\
             A stub nobody matched is the test's belief about the product's argv, and the \
             product disagreed. The call fell through to the empty-success default and the \
             assertions below it proved nothing — which is how the since-deleted `e2e_tests.rs` registered \
             `brew install {{name}}` against a product that emits `brew install -- neovim`, and \
             stayed green. Fix the pattern to the argv that ran, or call \
             `allow_unmatched_registrations()` and be able to say why.",
            unused.len(),
            unused.join("\n  "),
            if ran.is_empty() {
                "(nothing)".to_string()
            } else {
                ran.join("\n  ")
            }
        );
    }

    pub fn set_command_exists(&self, cmd: &str, exists: bool) {
        self.command_existence.insert(cmd.to_string(), exists);
    }

    pub async fn get_calls(&self) -> Vec<String> {
        self.call_log.lock().await.clone()
    }
}

#[async_trait]
impl ExecutionLayer for MockExecutor {
    async fn execute(
        &self,
        cmd: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<StdOutput> {
        let full_cmd = format!("{} {}", cmd, args.join(" "));
        {
            let mut log = self.call_log.lock().await;
            log.push(full_cmd.clone());
        }
        {
            let mut seen = self.last_env.lock().await;
            *seen = env.clone();
        }
        // A command that takes time, for the tests that are about *when* two calls run rather
        // than what they return. Without it a mock command is instantaneous, so contention on
        // `run_exclusive`'s per-backend mutex is unobservable and a test of lock granularity
        // passes identically against one global mutex.
        if let Some(d) = self.delays.get(&full_cmd).map(|d| *d.value()) {
            tokio::time::sleep(d).await;
        }
        if let Some(res) = self.responses.get(&full_cmd) {
            self.matched.insert(full_cmd, ());
            return res.clone();
        }
        if let Some(res) = self
            .default_response
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
        {
            return res.clone();
        }
        // Recorded, not refused. Most callers of an unstubbed command are asserting the argv or
        // the state afterwards and do not care what it printed; refusing here would redden them
        // all to catch the few that did care. `unmatched_registrations` is the half with no
        // innocent reading, and that one fails.
        self.unstubbed.insert(full_cmd, ());
        Ok(DryRunOutput::new().into())
    }

    /// Whether this command exists on the machine.
    ///
    /// Defaults to `true`, which is the same "silence means fine" choice as the empty output
    /// above and is kept for the same reason: a test that has not said otherwise is a test about
    /// something else, and a mock that answered `false` by default would have every backend
    /// report itself unavailable. The default is *recorded* so a test can ask.
    fn check_command(&self, cmd: &str) -> bool {
        match self.command_existence.get(cmd).map(|r| *r.value()) {
            Some(known) => known,
            None => {
                self.unstubbed.insert(format!("command -v {cmd}"), ());
                true
            }
        }
    }

    async fn symlink(&self, src: &Path, dst: &Path) -> Result<()> {
        let val = format!("LINK:{}", src.display());
        self.vfs.insert(dst.to_path_buf(), val);
        Ok(())
    }
}

impl Drop for MockExecutor {
    /// The check runs without a test having to remember it, which is the whole point: the tests
    /// that needed it were the ones whose authors did not know they did.
    ///
    /// Guarded on `thread::panicking` so a test already failing reports its own reason rather
    /// than this one — a second panic during unwind aborts the process and loses both messages.
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        // **Audit mode: report, never fail.** `SHALL_AUDIT_UNSTUBBED=1` turns every fall-through
        // in the run into one line on stdout instead of a panic, so the suite can be asked "how
        // many tests still leave a command unanswered?" without a per-test opt-in. It is the
        // measurement `#40` needs and it deliberately cannot fail a build: the conversion is
        // incremental and a global deny would redden a quarter of the suite for having no opinion.
        //
        // Here rather than inside the assertion below, which returns early for a mock that allowed
        // unmatched registrations — that early return would have hidden exactly the tests this
        // measures.
        //
        // Printed once per mock so one test cannot flood the log.
        if std::env::var_os("SHALL_AUDIT_UNSTUBBED").is_some()
            && !self
                .audited
                .swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            let unstubbed = self.unstubbed_registrations();
            if !unstubbed.is_empty() {
                println!(
                    "shall-unstubbed-audit: {} command(s) unanswered: {}",
                    unstubbed.len(),
                    unstubbed.join(" | ")
                );
            }
        }
        self.assert_every_registration_was_used();
    }
}
