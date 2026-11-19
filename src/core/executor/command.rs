//! The policy layer: sudo, per-manager locks, exit classification and the user-facing detail.

use super::*;

/// Strip inherited access and grant only the running user, via the tool Windows ships with.
///
/// Windows has no `mode` to create a file with, so "created restricted" is achieved by
/// restricting the temporary file and then renaming it into place — the destination never
/// exists in a readable state. A failure here is an error rather than a warning: the caller
/// is about to place a decrypted secret, and a secret that is not protected must not be
/// written at all (T5).
#[cfg(windows)]
fn restrict_to_owner(path: &Path) -> Result<()> {
    let user = std::env::var("USERNAME").map_err(|_| {
        Error::Refused(
            "cannot restrict the file: %USERNAME% is unset, so there is no account to grant \
             access to. Refusing to write a secret nothing protects."
                .into(),
        )
    })?;
    let output = StdCommand::new("icacls")
        .arg(path)
        .arg("/inheritance:r")
        .arg("/grant:r")
        .arg(format!("{}:F", user))
        .stdin(Stdio::null())
        .output()
        .map_err(|e| {
            Error::Other(format!(
                "could not run icacls to restrict {:?}: {}",
                path, e
            ))
        })?;
    if !output.status.success() {
        return Err(Error::Other(format!(
            "icacls could not restrict {:?}: {}",
            path,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(())
}

/// What one read-and-retry pass produced. `Answerless` carries the classified error rather
/// than having been returned, so the caller — and only the caller — decides whether "ran,
/// failed, said nothing" is a failure or a declared silence.
pub enum ReadOutcome {
    Output(String),
    Answerless(Error),
}

/// **Cloning shares the run, it does not fork it.** Every field below is an `Arc`, so a clone
/// is another handle on one invocation's execution layer, dry-run filesystem, per-manager lock
/// map and installed-listing memo — which is what lets ~48 backends each hold one and still
/// contend on the same locks and answer from the same memo. There used to be a `duplicate()`
/// beside this that was `self.clone()` and nothing else, so the codebase said both; the two
/// names carried one meaning and neither carried this note.
#[derive(Clone)]
pub struct CommandExecutor {
    pub dry_run: bool,
    pub verbose: bool,
    pub inner: Arc<dyn ExecutionLayer>,
    /// Where questions go. A search or an existence probe changes nothing, so it runs for
    /// real even under `--dry-run`: stubbing it does not make the preview safer, it makes
    /// the preview wrong. `apt-cache search jq` answered from a stub is an empty answer,
    /// which reads as "apt does not have jq" and hands the name to whichever manager
    /// answers over the network instead.
    pub reader: Arc<dyn ExecutionLayer>,
    vfs: Arc<DashMap<PathBuf, String>>,
    lock_map: Arc<DashMap<String, Arc<Mutex<()>>>>,
    /// What this backend's manager means by its exit codes and its complaints. Empty until a
    /// backend claims one with [`with_exit_policy`](Self::with_exit_policy), which is what
    /// keeps manager names out of this file.
    exit_policy: Arc<ExitPolicy>,
    /// This run's installed listings, one per manager.
    ///
    /// Lives here rather than in a global because every backend of one `App` is built on a
    /// duplicate of that `App`'s executor and every duplicate shares this `Arc` — so the memo
    /// is scoped to the run, which is also what keeps one test's mock listing out of the next
    /// test in a suite where a hundred `App`s live in one process.
    installed: Arc<crate::core::installed::InstalledListings>,
    /// What this run has established about escalating, and the answer it shares with every
    /// clone of this executor.
    ///
    /// **Same shape as `installed`, same reason.** A run asks the machine for credentials once;
    /// the answer belongs to the run. It was a process-wide `static`, which made it the one
    /// piece of run state every test in the binary shared — and a test that wanted to watch the
    /// credential probe was at the mercy of whichever test answered first (`#121`).
    sudo: Arc<SudoState>,
}

impl CommandExecutor {
    pub fn new(dry_run: bool, verbose: bool) -> Self {
        let vfs = Arc::new(DashMap::new());
        let lock_map = Arc::new(DashMap::new());
        let inner: Arc<dyn ExecutionLayer> = if dry_run {
            Arc::new(DryRunExecutor::new(vfs.clone()))
        } else {
            Arc::new(RawExecutor::mutator())
        };
        Self {
            dry_run,
            verbose,
            inner,
            reader: Arc::new(RawExecutor::reader()),
            vfs,
            lock_map,
            exit_policy: Arc::new(ExitPolicy::default()),

            installed: Arc::new(crate::core::installed::InstalledListings::new()),
            sudo: Arc::new(SudoState::new()),
        }
    }

    pub fn with_layer(
        dry_run: bool,
        verbose: bool,
        layer: Arc<dyn ExecutionLayer>,
        vfs: Arc<DashMap<PathBuf, String>>,
        lock_map: Arc<DashMap<String, Arc<Mutex<()>>>>,
    ) -> Self {
        // A test injects one layer and expects to see every call on it, reads included.
        Self {
            dry_run,
            verbose,
            reader: layer.clone(),
            inner: layer,
            vfs,
            lock_map,
            exit_policy: Arc::new(ExitPolicy::default()),

            installed: Arc::new(crate::core::installed::InstalledListings::new()),
            sudo: Arc::new(SudoState::new()),
        }
    }

    /// Let this run reuse installed listings written by earlier runs, for `secs` (0 = never).
    ///
    /// Set from config after the executor exists, rather than taken as a constructor argument:
    /// `CommandExecutor::new` is called from twenty-odd tests that have no config and want the
    /// off default, and a third constructor parameter would have to be spelled out in all of
    /// them to say "the same as before".
    pub fn set_installed_cache(&mut self, secs: u64) {
        if secs > 0 {
            self.installed = Arc::new(crate::core::installed::InstalledListings::with_ttl(secs));
        }
    }

    /// Bind this manager's exit conventions to the executor the backend will run on.
    ///
    /// Called at registration, beside the rest of that backend's definition, so a manager
    /// with a new convention is added by declaring one — never by editing this file.
    /// Can this executor's manager tell Shall that a name does not exist?
    ///
    /// Exposed so the property is *observable*: a backend that quietly lost its policy still
    /// runs the same argv, so nothing else can see the loss. `cargo` and `pipx` lost theirs in
    /// the 2026-08-04 conversion with every argv assertion green.
    pub fn classifies_absent_names(&self) -> bool {
        !self.exit_policy.absent_markers.is_empty()
    }

    pub fn with_exit_policy(mut self, policy: ExitPolicy) -> Self {
        self.exit_policy = Arc::new(policy);
        self
    }

    /// Whether a command asking for privilege actually gets a `sudo` in front of it here.
    ///
    /// **Three environments, two answers, and a test cannot guess which.** Windows never
    /// escalates; Linux as an ordinary user does; Linux as root does not, because it is already
    /// there. A test that hard-codes one of those is a test that passes on the machine it was
    /// written on — which is exactly what happened: three `web` tests registered `dpkg -r fd`,
    /// passed on Windows for months, and were never run on Linux at all because the build matrix
    /// silently produced one target out of four. On a Linux runner the product ran
    /// `sudo dpkg -r fd` and the stub went unmatched.
    ///
    /// So the rule is a function, and the tests ask it rather than restating it. One rule, one
    /// place; a second copy is the copy that is wrong on somebody else's machine.
    pub fn escalates(sudo: bool) -> bool {
        sudo && !cfg!(windows) && !Self::is_root()
    }

    /// The argv `run_on` would actually launch, as one string — what a mock registers against.
    ///
    /// **Public, and not `#[cfg(test)]`, for the reason [`escalates`](Self::escalates) is a
    /// function**: the integration suite registers stubs by this exact string, and a stub that
    /// spells the prefix by hand is a stub that goes unmatched the next time the prefix moves.
    /// It moved when `S88` put `-n` on every escalated command, and four registrations in
    /// `backend_tests.rs` would have silently stopped matching on Linux — the platform this
    /// machine cannot run, which is how the last set of unmatched stubs survived for months.
    pub fn as_launched(cmd: &str, args: &[&str], sudo: bool) -> String {
        let line = std::iter::once(cmd)
            .chain(args.iter().copied())
            .collect::<Vec<_>>()
            .join(" ");
        if Self::escalates(sudo) {
            // `-n` is part of the argv, so it is part of what a mock matches (`S88`). Leaving it
            // out here would make every Linux stub register a command Shall no longer runs.
            format!("sudo -n {line}")
        } else {
            line
        }
    }

    pub fn is_root() -> bool {
        #[cfg(unix)]
        {
            unsafe { libc::geteuid() == 0 }
        }
        #[cfg(windows)]
        {
            false
        }
    }

    /// Run a command and return its raw output WITHOUT enforcing exit status. Reads and
    /// existence probes use this (directly or via `run_output`), because a non-zero exit
    /// is frequently a normal answer there — an empty search, a "not installed" query, an
    /// inactive service unit. Mutating callers must use `run`/`run_exclusive` instead.
    async fn run_raw(&self, cmd: &str, args: &[&str], sudo: bool) -> Result<StdOutput> {
        self.run_on(&self.inner, cmd, args, sudo, None).await
    }

    /// The same primitive, aimed at the layer that never stubs. Reads only.
    async fn read_raw(&self, cmd: &str, args: &[&str], sudo: bool) -> Result<StdOutput> {
        self.run_on(&self.reader, cmd, args, sudo, None).await
    }

    pub(crate) async fn run_on(
        &self,
        layer: &Arc<dyn ExecutionLayer>,
        cmd: &str,
        args: &[&str],
        sudo: bool,
        as_account: Option<&Account>,
    ) -> Result<StdOutput> {
        let escalate = Self::escalates(sudo);

        // `apt install`, run by a sync that already holds the data-directory lock, fires the
        // `DPkg::Post-Invoke` hook Shall installed — which is another `shall`, and it would
        // wait on a lock this process does not release until it exits. The env var travels
        // to every descendant, and `hook-reconcile` stands down when it sees it.
        let mut env = HashMap::new();
        env.insert(
            crate::core::executor::INSIDE_SHALL.to_string(),
            std::process::id().to_string(),
        );
        Self::suppress_pagers(&mut env);

        if let Some(account) = as_account {
            for (key, value) in account.session_env() {
                env.insert(key, value);
            }
        }
        let target = self.account_target(cmd, as_account)?;

        if escalate {
            self.ensure_sudo_credentials(layer).await?;
        }

        let (final_cmd, final_args) = Self::account_argv(cmd, args, escalate, target, &env);

        // Every manager invocation funnels through this one call, which is what makes
        // `--timings` a breakdown of the whole run rather than of whichever verbs remembered
        // to instrument themselves.
        let timing = crate::core::timing::begin();
        let result = layer.execute(&final_cmd, &final_args, &env).await;
        crate::core::timing::end(timing, &final_cmd, &final_args);
        result
    }

    pub(crate) fn is_benign_exit(&self, code: Option<i32>) -> bool {
        self.exit_policy.is_benign(code)
    }

    pub(crate) fn can_become(&self) -> bool {
        #[cfg(unix)]
        {
            Self::is_root() && self.command_exists_sync("sudo")
        }
        #[cfg(not(unix))]
        false
    }

    /// Stop a child from piping itself into a pager.
    ///
    /// `systemctl status`, `git log` and friends page when they believe a human is watching.
    /// A pager waits for a keypress that a captured child will never get, so the run hangs;
    /// and even when it does not, the escape sequences and the `lines 1-16/16 (END)` banner
    /// land in the text a parser is about to read. Capturing stdout removes the usual trigger,
    /// but `$PAGER`/`$SYSTEMD_PAGER` in the user's environment forces one anyway — so the
    /// suppression is set here, on the one env map every spawn inherits, rather than trusted
    /// to the absence of a terminal.
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn suppress_pagers(env: &mut HashMap<String, String>) {
        // systemd reads an empty value as "no pager"; git and the rest need a command that
        // exists and exits, so `cat`.
        env.insert("SYSTEMD_PAGER".to_string(), String::new());
        env.insert("SYSTEMD_LESS".to_string(), String::new());
        env.insert("PAGER".to_string(), "cat".to_string());
        env.insert("GIT_PAGER".to_string(), "cat".to_string());
    }

    /// Run a *mutating* command and enforce success. `RawExecutor::execute` hands back the
    /// process output regardless of exit status, so without this a failed `apt remove` /
    /// `npm install` / `btrfs subvolume delete` would be silently reported as OK and the
    /// caller would trust a mutation that never actually happened. Callers that legitimately
    /// tolerate a non-zero exit (searches, existence probes) must use `run_output`/`run_raw`.
    pub async fn run(&self, cmd: &str, args: &[&str], sudo: bool) -> Result<StdOutput> {
        let output = self.run_raw(cmd, args, sudo).await?;
        let checked = self.ensure_status(cmd, output);
        // A mutation is the only thing that can change what is on PATH — or what is installed —
        // during a one-shot run. Installing `npm` and then asking whether `npm` is available is
        // a real sequence inside one `sync`. Every read stays memoised; a mutation is the edge
        // that invalidates them.
        self.forget_run_scoped_answers(None);
        checked
    }

    /// This run's installed listings, shared by every backend built on this executor.
    pub fn installed_listings(&self) -> &Arc<crate::core::installed::InstalledListings> {
        &self.installed
    }

    /// A read whose answer is its stdout.
    ///
    /// **A non-zero exit is tolerated; a non-zero exit that said nothing at all is not.** The
    /// distinction is the whole of it. "No such package" and "no results" are legitimate
    /// non-zero replies and they arrive with their reason on the page, so the exit code alone
    /// must never be the verdict — that is why this goes through the unchecked primitive rather
    /// than `run`. But a failure with nothing on *either* stream is not a reply at all, and
    /// returning `Ok("")` for it hands every caller an answer the manager never gave.
    ///
    /// Measured: 3 of 16 concurrent cold-start `winget list` exit `0x8A150001` having written
    /// zero bytes anywhere. Read as an empty listing, that made `shall list --backend winget`
    /// print nothing and exit 0 on a machine with 280 packages on it — and `info` report an
    /// installed package as absent, which is the shape it was first noticed in.
    ///
    /// **Retried when — and only when — the failure is classified transient.** Reads are
    /// idempotent, which is the whole justification: asking a manager what it has, twice,
    /// costs a second, where a mutation retried on a guess installs something twice. The
    /// measured case is a cold-start collision that a warm winget does not reproduce, so the
    /// second attempt is usually the entire fix.
    pub async fn run_output(&self, cmd: &str, args: &[&str], sudo: bool) -> Result<String> {
        match self.read_with_retry_as(None, cmd, args, sudo).await? {
            ReadOutcome::Output(s) => Ok(s),
            ReadOutcome::Answerless(e) => Err(e),
        }
    }

    /// [`Self::run_output`] with "the manager ran, failed, and said nothing" delivered as
    /// `Ok(None)` instead of an error — for the probes whose row declares that silence *is*
    /// the answer (`silence_is_none`).
    ///
    /// **The boundary is typed by construction, not matched out of prose.** The idle-timeout
    /// kill errors inside `read_with_retry` exactly as it does for `run_output`, so a wedged
    /// query killed at its bound arrives here as `Err` and propagates — it used to be
    /// indistinguishable from the answerless case because both said "no output", and a probe
    /// that timed out read as "zero updates", exit 0.
    pub async fn run_output_maybe_silent(
        &self,
        cmd: &str,
        args: &[&str],
        sudo: bool,
    ) -> Result<Option<String>> {
        match self.read_with_retry_as(None, cmd, args, sudo).await? {
            ReadOutcome::Output(s) => Ok(Some(s)),
            ReadOutcome::Answerless(_) => Ok(None),
        }
    }

    /// The one read-and-retry loop both output primitives share. An answerless read — non-zero
    /// exit, both streams empty — retries while transient and then becomes
    /// [`ReadOutcome::Answerless`], carrying the classified error for whoever decides whether
    /// that means failure or silence.
    pub(crate) async fn read_with_retry_as(
        &self,
        account: Option<&Account>,
        cmd: &str,
        args: &[&str],
        sudo: bool,
    ) -> Result<ReadOutcome> {
        let attempts = read_retry_attempts();
        let initial = std::time::Duration::from_millis(200);
        // The same cap the mutation retry uses (`TransactionConfig::max_backoff`): a cap is
        // not a mutation-only concern. Uncapped, ten attempts slept ~39s before the last
        // try — for a read, where the whole justification is that a second attempt usually
        // fixes it.
        let max = std::time::Duration::from_secs(30);
        let mut backoff = initial;
        for attempt in 1..=attempts {
            let output = match account {
                Some(account) => self.read_raw_as(account, cmd, args, sudo).await?,
                None => self.read_raw(cmd, args, sudo).await?,
            };
            let stdout = crate::utils::text::sanitize(&String::from_utf8_lossy(&output.stdout));
            let benign =
                output.status.success() || self.exit_policy.is_benign(output.status.code());
            // **Both streams, not just stdout.** A read that failed and *said* something has
            // described its own situation, and the caller may legitimately read that as an
            // empty result: `Get-ComputerRestorePoint` on an unelevated shell exits 1 with
            // `Access denied` on stderr and nothing on stdout, and treating that as fatal
            // failed a whole `sync` that had no business caring. Whether *that* is the right
            // answer is a separate question about snapshots, not about this primitive.
            //
            // Silence on both is the case with no second reading. Nothing expresses "you have
            // none of these" by saying nothing at all and failing.
            let said_nothing = stdout.trim().is_empty()
                && String::from_utf8_lossy(&output.stderr).trim().is_empty();
            if benign || !said_nothing {
                return Ok(ReadOutcome::Output(stdout));
            }
            let err = self.answerless_read(cmd, args, &output);
            if attempt == attempts || err.retryability() != Retryability::Transient {
                return Ok(ReadOutcome::Answerless(err));
            }
            debug!(
                "`{cmd}` produced no answer and the failure is transient; \
                 asking again ({attempt}/{attempts})"
            );
            tokio::time::sleep(backoff).await;
            backoff = std::cmp::min(backoff * 3, max);
        }
        unreachable!("the loop returns on its last attempt")
    }

    /// A read that must distinguish "this manager will not do that" from "it did it and found
    /// nothing" — the question `run_output` deliberately refuses to answer.
    ///
    /// Used to ask a manager for a machine-readable listing it may be too old to support
    /// (`Q43`). An unsupported flag exits non-zero with a usage message, which every other
    /// reader here hands back as an empty result — correct for them, and the exact bug `Q40`
    /// closed if a caller then treats it as the listing. So negotiation gets its own primitive
    /// rather than a looser rule for everybody.
    ///
    /// Not retried and not classified: the answer being sought *is* the failure.
    pub async fn probe_output(&self, cmd: &str, args: &[&str]) -> Result<String> {
        let output = self.read_raw(cmd, args, false).await?;
        if !output.status.success() && !self.exit_policy.is_benign(output.status.code()) {
            return Err(self.answerless_read(cmd, args, &output));
        }
        Ok(crate::utils::text::sanitize(&String::from_utf8_lossy(
            &output.stdout,
        )))
    }

    /// The failure for a read that exited non-zero without producing an answer.
    ///
    /// Carries the manager's own words when it left any, and its retryability, so the retry
    /// loop does not have to read the sentence back to decide what to do.
    pub(crate) fn answerless_read(&self, cmd: &str, args: &[&str], output: &StdOutput) -> Error {
        let code = output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "terminated by signal".to_string());
        let stderr = crate::utils::text::sanitize(&String::from_utf8_lossy(&output.stderr));
        let said = stderr
            .trim()
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        let detail = if said.is_empty() {
            String::new()
        } else {
            format!(" It said: {said}")
        };
        let hay = ExitPolicy::haystack(&output.stdout, &output.stderr);
        Error::CommandFailed {
            message: format!(
                "`{} {}` exited {code} with no output, so Shall has no answer from it — not an \
                 empty one.{detail}",
                cmd,
                args.join(" ")
            ),
            // By code as well as by text. This is the one failure whose haystack is reliably
            // empty, so the text lists cannot classify it and the code is the only signal
            // there is.
            retry: self.exit_policy.retryability_of(output.status.code(), &hay),
            absent_name: false,
        }
    }

    /// A read whose emptiness is an *answer*, so a command that could not produce one must
    /// say so instead of returning nothing.
    ///
    /// "This manager has no such package" and "this manager has no package index" both print
    /// nothing. Reading the second as the first is how a bare name walks past the manager
    /// that has it and freezes to a lower one (V.7c). A non-zero exit alone is not the
    /// signal — `pacman -Ss`, `dnf search` and `brew search` all exit non-zero for an
    /// ordinary empty result — so the fault is a non-zero exit *with* a complaint on stderr.
    pub async fn search_output(&self, cmd: &str, args: &[&str], sudo: bool) -> Result<String> {
        let output = self.read_raw(cmd, args, sudo).await?;
        let stderr = crate::utils::text::sanitize(&String::from_utf8_lossy(&output.stderr));
        let complaint = stderr.trim();
        if !output.status.success() && !complaint.is_empty() {
            let first = complaint.lines().next().unwrap_or(complaint);
            // Not `CommandFailed`: this sentence is read by a user, in a line that
            // already says which manager and which package, and "Command execution
            // failed:" in front of it is noise.
            return Err(Error::Other(format!(
                "`{}` could not answer: {}",
                cmd, first
            )));
        }
        Ok(crate::utils::text::sanitize(&String::from_utf8_lossy(
            &output.stdout,
        )))
    }

    /// The file behind `run_exclusive`'s cross-process lock, in Shall's own data directory.
    ///
    /// It lived at a fixed, guessable name in the shared temp directory and was opened with
    /// `File::create`, which truncates and follows symlinks — so anyone with write access to
    /// that directory could pre-plant `shall_apt.lock` as a symlink and have the next
    /// exclusive run, frequently privileged, truncate the target. `datalock.rs` had already
    /// solved this; this is the same treatment, so there is one locking style in the tree and
    /// not two.
    fn open_exec_lock(lock_key: &str) -> Result<File> {
        Self::open_lock_at(&crate::utils::safe_data_dir().join("exec-locks"), lock_key)
    }

    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn open_lock_at(dir: &Path, lock_key: &str) -> Result<File> {
        // Created once per process, not once per lock: this runs on every exclusive command,
        // for a directory that exists after the first one.
        if LOCK_DIRS.insert(dir.to_path_buf()) {
            crate::utils::file::ensure_dir(dir)?;
        }
        // A lock key is a backend name, and a backend name comes from a config file. Anything
        // that is not a plain name would otherwise pick the directory the lock lands in.
        let stem: String = lock_key
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let path = dir.join(format!("{}.lock", stem));
        let open = || {
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&path)
        };
        match open() {
            Ok(file) => Ok(file),
            // **The cache above is an optimisation, and it must not be load-bearing.** It records
            // that a directory was created, not that it still exists — and `create(true)` creates
            // the *file*, never its parent. So a data directory removed after the first exclusive
            // command of the process fails every one after it with a bare "cannot find the path
            // specified", which names neither the directory nor the fact that it went missing.
            //
            // Measured as a Windows CI flake: the data directory is a process-global setting, the
            // test suite points it at temporary directories, and a directory cached by one test
            // was deleted while another still held the memo. That is a test arrangement, but the
            // mechanism is not — `/tmp` reapers and cleanup tools do the same thing to a long
            // `sync`, and the answer in both cases is to make the directory again rather than to
            // trust a memo about the past.
            Err(_) => {
                crate::utils::file::ensure_dir(dir)?;
                open().map_err(Error::from)
            }
        }
    }

    pub async fn run_exclusive(
        &self,
        lock_key: &str,
        cmd: &str,
        args: &[&str],
        sudo: bool,
    ) -> Result<StdOutput> {
        let mutex = self
            .lock_map
            .entry(lock_key.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();
        let _thread_guard = mutex.lock().await;

        if self.dry_run {
            return self.run(cmd, args, sudo).await;
        }

        let lock_file = Self::open_exec_lock(lock_key)?;
        // `fs2`'s flock is a blocking syscall. When a second `shall` holds this lock, taking it
        // inline parks a whole runtime worker for however long that other process runs —
        // minutes, for an `apt dist-upgrade`. The wait belongs on the blocking pool; the file
        // comes back out so the unlock below is the same handle.
        let lock_file =
            tokio::task::spawn_blocking(move || lock_file.lock_exclusive().map(|_| lock_file))
                .await
                .map_err(|e| Error::Io(format!("waiting for the `{}` lock: {}", lock_key, e)))?
                .map_err(Error::from)?;
        let result = self.run_raw(cmd, args, sudo).await;
        let _ = lock_file.unlock();
        // Same invalidation as `run`, and it has to be here too: this is the path most installs
        // and removals actually take, and it reaches `run_raw` directly rather than through
        // `run`. Without it the run-scoped listings would still be answering from before the
        // install — which `Prior` reads to decide what a rollback puts back.
        self.forget_run_scoped_answers(Some(lock_key));
        // Enforce status only after releasing the lock, so a failed mutation still frees it.
        self.ensure_status(cmd, result?)
    }

    /// Forget everything memoised for the length of a run that a mutation could have changed:
    /// what is on `PATH`, and what each manager has installed.
    pub(crate) fn forget_run_scoped_answers(&self, manager: Option<&str>) {
        forget_path_lookups();
        self.installed.forget_after_mutation(manager);
    }

    /// What a failed command's own output is allowed to put on a terminal.
    ///
    /// A manager's stream is untrusted text of unbounded length: one `scoop` typo produced ~110
    /// lines of unrelated bucket commits with raw SGR sequences, and the sentence that mattered
    /// — `Couldn't find manifest for 'x'` — was the fourth of them. The escapes were already
    /// handled for the *machine* (`ExitPolicy::opening` strips them so detection works) and not
    /// for the person, which is the wrong way round.
    ///
    /// So: the manager's own vocabulary picks the lines that explain the failure; failing that,
    /// the tail, because a tool that says nothing it declared usually ends with its complaint.
    /// Whatever is dropped is named and reachable — the whole stream is logged at `-v` by the
    /// caller, and a count with nowhere to look is not one place to look.
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn detail_for_user(policy: &ExitPolicy, stream: &str) -> String {
        /// Enough for a stack trace's opening, far short of a bucket update.
        const CAP: usize = 8;

        let lines: Vec<&str> = stream
            .lines()
            .map(str::trim_end)
            .filter(|l| !l.trim().is_empty())
            .collect();
        if lines.is_empty() {
            return String::new();
        }
        let explaining = policy.explaining_lines(stream);
        let chosen: Vec<&str> = if !explaining.is_empty() {
            explaining.into_iter().take(CAP).collect()
        } else if lines.len() > CAP {
            lines[lines.len() - CAP..].to_vec()
        } else {
            lines.clone()
        };
        /// A line long enough for any sentence a manager writes, and short enough that a stream
        /// with no newlines in it — a progress bar drawn with bare carriage returns is one long
        /// line to `lines()` — cannot fill a terminal past the cap above.
        const WIDTH: usize = 240;

        let shown: Vec<String> = chosen
            .iter()
            // A tab is a column, not an escape: kept as spaces so a manager's table still reads
            // as one. Everything else that moves a cursor or reverses a line is named by
            // codepoint, by the same function the grammar's refusals use.
            .map(|l| crate::core::validator::printable(&l.replace('\t', "    ")))
            .map(|l| {
                if l.chars().count() > WIDTH {
                    let head: String = l.chars().take(WIDTH).collect();
                    format!("{head}…")
                } else {
                    l
                }
            })
            .collect();
        let dropped = lines.len().saturating_sub(shown.len());
        let mut out = shown.join("\n");
        if dropped > 0 {
            out.push_str(&format!(
                "\n({} more line(s) of output; re-run with -v to see all of it)",
                dropped
            ));
        }
        out
    }

    /// Classify a finished mutating command as success or failure, and — when it failed —
    /// whether another attempt could go differently.
    ///
    /// What a non-zero exit or a zero-exit complaint means belongs to the manager, so it is
    /// read off this executor's [`ExitPolicy`] rather than matched on the program's name
    /// here. An executor with no policy is the honest default: every non-zero exit fails and
    /// nothing is classified, which is what the retry loop already assumed.
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    pub fn ensure_status(&self, cmd: &str, output: StdOutput) -> Result<StdOutput> {
        let status_ok = output.status.success() || self.exit_policy.is_benign(output.status.code());
        // One lowercased join of both streams for all three marker questions below. Each used
        // to build its own, so a command's whole transcript was copied and lowercased three
        // times — per package, and an `apt install` or `cargo build` transcript is not small.
        // A policy with no markers answers every question from its empty lists and never reads
        // this, so it is not built for one.
        let hay = if self.exit_policy.reads_output() {
            ExitPolicy::haystack(&output.stdout, &output.stderr)
        } else {
            String::new()
        };
        if status_ok && !self.exit_policy.signals_failure(&hay) {
            return Ok(output);
        }
        let code = output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "terminated by signal".to_string());
        // scoop's failure marker lands on stdout, not stderr, so fall back to stdout for
        // the diagnostic when stderr is empty (e.g. a `status_ok` malignant-success case).
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = crate::utils::text::sanitize(&String::from_utf8_lossy(&output.stdout));
        let stream = {
            let e = stderr.trim();
            if e.is_empty() {
                stdout.trim()
            } else {
                e
            }
        };
        // Everything the command said, for whoever asked for the internals. The message below
        // keeps only what explains the failure, so this is the one place the rest survives.
        if !stream.is_empty() {
            debug!("`{}` said:\n{}", cmd, stream);
        }
        let detail = Self::detail_for_user(&self.exit_policy, stream);
        // A manager that exited 0 and said it failed is not described by its exit code.
        // "`scoop` failed (exit 0)" is a sentence that argues with itself, and it was the
        // first thing a user saw after a typo; when the verdict comes from what the command
        // printed, the message says that instead of quoting a status that means the opposite.
        let verdict = if output.status.success() {
            format!("`{}` reported a failure", cmd)
        } else {
            format!("`{}` failed (exit {})", cmd, code)
        };
        let msg = if detail.is_empty() {
            verdict
        } else {
            format!("{}: {}", verdict, detail)
        };
        Err(Error::CommandFailed {
            message: msg,
            retry: self.exit_policy.retryability(&hay),
            // Asked of the whole output, here, because this is the last place it exists: the
            // message keeps whichever stream was non-empty, so a marker on the other one is
            // gone by the time any caller sees it.
            absent_name: self.exit_policy.names_an_absent_package(&hay),
        })
    }

    pub async fn read_file(&self, path: &Path) -> Result<String> {
        if self.dry_run {
            if let Some(content) = self.vfs.get(path) {
                return Ok(content.clone());
            }
        }
        tokio::fs::read_to_string(path).await.map_err(Error::from)
    }

    /// Write a file the *machine* owns — a systemd unit, a `link:` target, a backend's state
    /// file — atomically and durably.
    ///
    /// The executor's preview policy, not [`crate::utils::file::persist`]'s: a dry run diverts
    /// the bytes into the VFS so a later read in the same run sees what this run would have
    /// written. Both policies are legitimate and they answer different questions; only the
    /// durability is shared.
    ///
    /// **It had no `flush` and no `sync_all`.** A rename is atomic against a concurrent reader
    /// and says nothing about power loss, so a crash after a sync could leave a zero-length
    /// systemd unit while `registry.json` and the WAL — which go through `persist` — survived
    /// intact. Shall's record of what it did, without the thing it did.
    pub async fn write_atomic(&self, path: &Path, content: &str) -> Result<()> {
        if self.dry_run {
            self.vfs.insert(path.to_path_buf(), content.to_string());
            return Ok(());
        }
        Self::durably(path, content, |_| Ok(())).await
    }

    /// Remove a file Shall placed, recording nothing in a dry run.
    ///
    /// A dry run does not delete, and it also does not *pretend* to: inserting a marker into
    /// the VFS would make later dry-run reads of this path answer from fiction. Already-gone
    /// is done, which is what makes teardown idempotent.
    pub async fn remove_file(&self, path: &Path) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }
        match tokio::fs::remove_file(path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// The one place this crate's async writers meet the one durable write.
    ///
    /// `spawn_blocking` because `sync_all` parks a thread on the disk, and parking a runtime
    /// worker there stalls every other task on it.
    async fn durably(
        path: &Path,
        content: &str,
        prepare: impl FnOnce(&Path) -> Result<()> + Send + 'static,
    ) -> Result<()> {
        let path = path.to_path_buf();
        let content = content.to_string();
        tokio::task::spawn_blocking(move || {
            crate::utils::file::durable_write(&path, &content, prepare)
        })
        .await
        .map_err(|e| Error::Other(format!("IO thread failure: {}", e)))?
    }

    /// Write content that must never be world-readable, restricted **before** it reaches its
    /// destination (T5).
    ///
    /// The restriction is applied to the temporary file and the file is then renamed into
    /// place, so there is no instant at which the target path holds readable plaintext. A
    /// chmod after the write would be that instant, however short — and a secret is exactly
    /// the file where "however short" is not an argument.
    ///
    /// On Unix the temp file is already created `0600` by `tempfile`; this asserts it rather
    /// than assuming it. On Windows the inherited ACEs are stripped and only the running user
    /// is granted access, via `icacls` — Shall drives the tool the OS already has.
    pub async fn write_secret(&self, path: &Path, content: &str) -> Result<()> {
        if self.dry_run {
            self.vfs.insert(path.to_path_buf(), content.to_string());
            return Ok(());
        }
        Self::durably(path, content, |temp| {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(temp, std::fs::Permissions::from_mode(0o600))
                    .map_err(Error::from)?;
            }
            #[cfg(windows)]
            {
                restrict_to_owner(temp)?;
            }
            Ok(())
        })
        .await
    }

    pub async fn symlink(&self, src: &Path, dst: &Path) -> Result<()> {
        self.inner.symlink(src, dst).await
    }

    pub fn get_vfs_diff(&self) -> Vec<(PathBuf, String)> {
        self.vfs
            .iter()
            .map(|item| (item.key().clone(), item.value().clone()))
            .collect()
    }

    pub async fn command_exists(&self, cmd: &str) -> bool {
        self.reader.check_command(cmd)
    }

    pub fn command_exists_sync(&self, cmd: &str) -> bool {
        self.reader.check_command(cmd)
    }

    /// Make sure `sudo` will run without asking, or fail **now** with a sentence saying why
    /// (`S88`).
    ///
    /// **The bug this ends.** sudo reads its password from `/dev/tty`, not from stdin, so
    /// nothing Shall did to the child's stdin could stop it waiting: a wrong password left the
    /// program sitting for 900 seconds, and so did a terminal with nobody in front of it. Both
    /// were silent — no prompt reached the user, because the child's stderr is captured — and
    /// both looked exactly like a slow package manager. It failed that way every night for six
    /// nights in the `tools` leg, and took 48 minutes to do it.
    ///
    /// **Three outcomes, all of them prompt.** The timestamp is already warm, so nothing is
    /// asked; a person is at a terminal and gets one bounded chance to type; or there is nobody
    /// to ask and Shall says so immediately instead of waiting to be told what it already knows.
    async fn ensure_sudo_credentials(&self, layer: &Arc<dyn ExecutionLayer>) -> Result<()> {
        if cfg!(windows) || Self::is_root() || self.dry_run {
            return Ok(());
        }
        if self.sudo.is_primed() {
            return Ok(());
        }
        // **The answer is remembered in both directions** (`S89`). A refusal is as permanent
        // within one run as a success is, and asking again costs the whole password bound again
        // — per escalated command, on the verbs that deliberately keep going after one backend
        // fails. That is how a 120-second bound produced a 900-second wedge.
        if let Some(why) = self.sudo.refusal() {
            return Err(Error::command_failed_permanently(why));
        }

        // Everything above is a fast path off an already-settled answer. Past here the call
        // may spawn a probe and may put a prompt on the terminal, so only one task does it —
        // and both cells are re-read after the wait, because the task that held the lock has
        // just written whichever of them applies.
        let _priming = self.sudo.priming().await;
        if self.sudo.is_primed() {
            return Ok(());
        }
        if let Some(why) = self.sudo.refusal() {
            return Err(Error::command_failed_permanently(why));
        }
        // Warm timestamp, `NOPASSWD`, or an already-primed session — and **through the layer, not
        // around it**: this spawned `Command::new("sudo")`, the one command `with_layer` could not
        // see, so an escalation test needed passwordless sudo on the host to pass (`#88`). A layer
        // can now *answer* the probe.
        let probe = ["-n".to_string(), "-v".to_string()];
        let warm = layer.execute("sudo", &probe, &HashMap::new()).await;
        if matches!(warm, Ok(out) if out.status.success()) {
            self.sudo.mark_primed();
            return Ok(());
        }

        // Nobody to ask. **Permanent, not transient**: a password does not arrive during a
        // backoff, and retrying spends the whole bound again to learn the same thing.
        if !(layer.shares_stdin() && std::io::stdin().is_terminal()) {
            return Err(self.sudo.refuse(
                "sudo needs a password and there is no terminal to ask on. Run Shall from a \
                 terminal, give this user a NOPASSWD rule for the package managers it drives, \
                 or run it as root."
                    .to_string(),
            ));
        }

        let ask = Command::new("sudo")
            // **The one child that must stay in Shall's own process group.** Every other child
            // leads its own so `Stopping` can signal the tree, and such a child is stopped at
            // its first read of a terminal it does not own (`#132`, see `supervise::Foreground`).
            // This spawn *is* the read — it exists to put a password prompt on the terminal — so
            // grouping it would turn the one prompt Shall allows itself into that same silent
            // stop. Its bound is `sudo_password_timeout_secs`, not a process group.
            .arg("-v")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .status();
        let answered = match sudo_password_timeout() {
            Some(bound) => match tokio::time::timeout(bound, ask).await {
                Ok(status) => status,
                Err(_) => {
                    return Err(self.sudo.refuse(format!(
                        "sudo asked for a password and none was entered within {}s. Set \
                         `sudo_password_timeout_secs` to wait longer, or `0` to wait as long as \
                         sudo would.",
                        bound.as_secs()
                    )))
                }
            },
            None => ask.await,
        };
        match answered {
            Ok(status) if status.success() => {
                self.sudo.mark_primed();
                Ok(())
            }
            // sudo has already printed its own reason — a wrong password, an account that is
            // not a sudoer — on the terminal it owns. Repeating a guess at it here would be
            // narration over a message the user has already read.
            Ok(_) => Err(self
                .sudo
                .refuse("sudo refused: Shall cannot run the commands that need root.".to_string())),
            Err(e) => Err(self.sudo.refuse(format!("sudo could not be run: {e}"))),
        }
    }

    /// Refresh the `sudo` timestamp for as long as the returned guard is held, so a long sync
    /// is not interrupted halfway by a password prompt.
    ///
    /// The guard is the whole point: the previous version handed back a bare `JoinHandle`,
    /// which detaches when dropped, so the loop outlived every caller and could not be
    /// stopped by any of them.
    pub async fn start_sudo_keepalive(&self) -> SudoKeepalive {
        if cfg!(windows) || Self::is_root() || self.dry_run {
            return SudoKeepalive(None);
        }
        // A machine that has already recorded a permanent refusal will not start answering
        // `sudo -n -v`, so refreshing a timestamp that does not exist is one pointless process
        // every sixty seconds for the rest of the run.
        if self.sudo.refusal().is_some() {
            return SudoKeepalive(None);
        }
        SudoKeepalive(Some(tokio::spawn(async move {
            loop {
                // `-n` so this never prompts. The foreground command owns the terminal; a
                // background task racing it for the same password prompt is two processes
                // reading one keyboard, and the visible one loses. An expired timestamp is
                // the foreground command's to raise, where the user can see which command
                // is asking.
                let _ = Command::new("sudo")
                    .args(["-n", "-v"])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .kill_on_drop(true)
                    .status()
                    .await;
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        })))
    }
}

/// Stops the `sudo -v` loop when it goes out of scope.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so a private item is invisible there.
pub struct SudoKeepalive(pub Option<tokio::task::JoinHandle<()>>);

impl SudoKeepalive {
    /// Whether a refresher is actually running — false on Windows, as root, and under
    /// `--dry-run`, where there is no timestamp to keep warm.
    pub fn is_running(&self) -> bool {
        self.0.as_ref().is_some_and(|h| !h.is_finished())
    }
}

impl Drop for SudoKeepalive {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.abort();
        }
    }
}
