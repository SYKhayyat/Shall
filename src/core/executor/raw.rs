//! The real process runner: spawn, pump, and drain the pipes without wedging on a silence.

use super::*;

pub struct RawExecutor {
    /// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
    /// test crate links the library without `cfg(test)`, so a private item is invisible there.
    pub stdin: ChildStdin,
    pub idle: Option<std::time::Duration>,
}

impl RawExecutor {
    /// The layer behind `run_output`/`search_output`/`command_exists`.
    ///
    /// Bounded by `query_idle_timeout`, not the mutation bound. A question that has said
    /// nothing for two minutes is not about to answer, and making it wait out a bound sized
    /// for `Checkpoint-Computer` buys nothing but the wait.
    pub fn reader() -> Self {
        Self {
            stdin: ChildStdin::Closed,
            idle: query_idle_timeout(),
        }
    }

    /// The layer behind `run`/`run_exclusive`.
    ///
    /// **`Interactive` is a statement about the person, not about the child's handles.** It
    /// means a mutation may have to prime `sudo` once and its output is worth mirroring to the
    /// terminal as it arrives; what the child itself is given is `execute`'s decision, and on
    /// Unix that is no terminal at all (`#132`). The measurement that closed Windows' half
    /// still stands: 48ms with stdin closed, 21.9s with a real console — at the shipped bound, a
    /// fifteen-minute silence ending in failure anyway.
    pub fn mutator() -> Self {
        Self {
            stdin: if cfg!(windows) {
                ChildStdin::Closed
            } else {
                ChildStdin::Interactive
            },
            idle: command_idle_timeout(),
        }
    }

    /// **Ships, and it is not C3's case.** The constructor mints no authority — it sets two
    /// fields an executor already has — so a test building an executor with an eager idle
    /// is a caller like any other, not one that has to stay behind `cfg(test)`.
    pub fn with_idle(stdin: ChildStdin, idle: Option<std::time::Duration>) -> Self {
        Self { stdin, idle }
    }

    /// How long a child gets to stop itself before it is killed outright.
    ///
    /// Long enough for a package manager to abort a transaction and unlink its lock, short
    /// enough that a run Shall has already given up on does not sit there. `dpkg`'s own
    /// shutdown path is the slowest of these and finishes well inside it.
    ///
    /// Unix only, because it is the grace between the signal that can be caught and the one that
    /// cannot, and Windows has only the second.
    #[cfg(unix)]
    pub(crate) const TERMINATION_GRACE: std::time::Duration = std::time::Duration::from_secs(10);

    /// Collect the child's output, optionally echoing it to the terminal as it arrives, and
    /// give up on a child that has gone silent.
    ///
    /// Both streams must be drained concurrently with the wait: a pipe buffer that fills while
    /// nothing reads it blocks the child forever, and a package manager writing more than the
    /// buffer holds is not an edge case.
    ///
    /// The wait is sliced rather than awaited whole so the child stays reachable between
    /// slices — killing it needs the same `&mut` the wait future holds.
    ///
    /// The child is stopped through [`Stopping`], never with a bare kill: what Shall spawns is
    /// usually a package manager, and how you stop one of those decides whether the machine is
    /// usable afterwards.
    pub(crate) async fn wait_watched(
        mut child: tokio::process::Child,
        cmd: &str,
        mirror: bool,
        idle: Option<std::time::Duration>,
    ) -> Result<StdOutput> {
        use std::sync::Mutex as SyncMutex;
        use std::time::Instant;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        async fn pump<R: tokio::io::AsyncRead + Unpin>(
            mut src: R,
            mirror: bool,
            last: Arc<SyncMutex<Instant>>,
        ) -> std::io::Result<Vec<u8>> {
            let mut collected = crate::core::capture::Capped::new();
            let mut buf = [0u8; 8192];
            let mut sink = tokio::io::stderr();
            loop {
                let n = src.read(&mut buf).await?;
                *last.lock().unwrap_or_else(|e| e.into_inner()) = Instant::now();
                if n == 0 {
                    return Ok(collected.finish());
                }
                collected.push(&buf[..n]);
                // Reading never stops at the cap, only *keeping* does: a child whose output
                // nobody drained blocks on a full pipe forever, which is the failure the
                // concurrent drain above exists to prevent.
                if mirror {
                    sink.write_all(&buf[..n]).await?;
                    sink.flush().await?;
                }
            }
        }

        let last: Arc<SyncMutex<Instant>> = Arc::new(SyncMutex::new(Instant::now()));
        let out_pipe = child.stdout.take();
        let err_pipe = child.stderr.take();
        // From here the child is owned by the guard: every way out of this function — returning,
        // an error, or the whole future being dropped by an aborted worker — goes through it.
        let mut child = Stopping::new(child);
        let mut out_task = tokio::spawn({
            let last = last.clone();
            async move {
                match out_pipe {
                    Some(p) => pump(p, mirror, last).await,
                    None => Ok(Vec::new()),
                }
            }
        });
        let mut err_task = tokio::spawn({
            let last = last.clone();
            async move {
                match err_pipe {
                    Some(p) => pump(p, mirror, last).await,
                    None => Ok(Vec::new()),
                }
            }
        });

        let status = match idle {
            None => child.child.wait().await?,
            Some(idle) => loop {
                let quiet = last.lock().unwrap_or_else(|e| e.into_inner()).elapsed();
                let remaining = idle.saturating_sub(quiet);
                if remaining.is_zero() {
                    child.stop().await;
                    out_task.abort();
                    err_task.abort();
                    // Permanent: the retry loop would spend another `idle` per attempt on a
                    // command that has already proved it does not finish, and a user watching
                    // three silences instead of one learns nothing new from the second two.
                    return Err(Error::command_failed_permanently(format!(
                        "`{}` produced no output for {}s and had not exited; Shall asked it to \
                         stop and killed it if it would not. If this command is legitimately \
                         silent for longer, raise `command_idle_timeout_secs` (0 disables the \
                         bound).",
                        cmd,
                        idle.as_secs(),
                    )));
                }
                if let Ok(status) = tokio::time::timeout(remaining, child.child.wait()).await {
                    break status?;
                }
            },
        };
        // The child has exited; its *output* may not have. A manager that hands stdout to a
        // background process and returns leaves a pipe held by something outside this process
        // tree — `child.wait()` is done, there is nothing left to kill, and an unclocked
        // `await` here waits on an EOF that never comes. Measured: a 20s bound, a 64s wall,
        // and the install reported as a success. So the same clock keeps running over the
        // readers, on silence rather than duration, so a command still printing is never cut.
        let mut stalled = false;
        let out = Self::drain_watched(&mut out_task, &last, idle, &mut stalled).await;
        let err = Self::drain_watched(&mut err_task, &last, idle, &mut stalled).await;
        if stalled {
            out_task.abort();
            err_task.abort();
            return Err(Error::command_failed_permanently(format!(
                "`{}` exited, but something still holds its output open and has printed \
                 nothing for {}s; Shall stopped waiting. This is what a command that hands \
                 its work to a background process looks like from here. If it is legitimately \
                 silent for longer, raise `command_idle_timeout_secs` (0 disables the bound).",
                cmd,
                idle.map_or(0, |d| d.as_secs()),
            )));
        }
        let joined = |r: std::result::Result<std::io::Result<Vec<u8>>, tokio::task::JoinError>| {
            r.map_err(|e| Error::Other(format!("output reader failed: {}", e)))?
                .map_err(Error::from)
        };
        Ok(StdOutput {
            status,
            stdout: joined(out.expect("not stalled, so the reader finished"))?,
            stderr: joined(err.expect("not stalled, so the reader finished"))?,
        })
    }

    /// Await one output reader under the same silence bound the wait used.
    ///
    /// `stalled` latches: once either reader has gone quiet past the bound the other is not
    /// waited on at all, because the failure is already decided and waiting again would spend
    /// a second `idle` proving it.
    async fn drain_watched(
        task: &mut tokio::task::JoinHandle<std::io::Result<Vec<u8>>>,
        last: &Arc<std::sync::Mutex<std::time::Instant>>,
        idle: Option<std::time::Duration>,
        stalled: &mut bool,
    ) -> Option<std::result::Result<std::io::Result<Vec<u8>>, tokio::task::JoinError>> {
        if *stalled {
            return None;
        }
        let Some(idle) = idle else {
            return Some(task.await);
        };
        loop {
            let quiet = last.lock().unwrap_or_else(|e| e.into_inner()).elapsed();
            let remaining = idle.saturating_sub(quiet);
            if remaining.is_zero() {
                *stalled = true;
                return None;
            }
            if let Ok(done) = tokio::time::timeout(remaining, &mut *task).await {
                return Some(done);
            }
        }
    }
}

#[async_trait]
impl ExecutionLayer for RawExecutor {
    async fn execute(
        &self,
        cmd: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<StdOutput> {
        // On Windows, route shim scripts (scoop's `.ps1`, `.cmd`/`.bat` wrappers) through
        // their interpreter so they can actually launch.
        #[cfg(windows)]
        let (eff_cmd, eff_args) = windows_effective_command(cmd, args);
        #[cfg(windows)]
        let (cmd, args) = (eff_cmd.as_str(), eff_args.as_slice());

        let mut command = Command::new(cmd);
        command.args(args).envs(env);

        // **Pinned off the directory Shall was invoked from.** A "global" install resolved
        // against the caller's CWD inherits whatever project files live there: a `.npmrc`
        // three directories up redirects the install, a `.cargo/config.toml` swaps the
        // registry, and the manager answers for a machine nobody configured on purpose.
        // Managers are invoked here in their machine-wide capacity or not at all, so they run
        // from the neutral temp directory — no project above it to be discovered by accident.
        {
            let neutral = std::env::temp_dir();
            if neutral.is_dir() {
                command.current_dir(neutral);
            }
        }

        // A worker whose task is aborted — a failed node, the global timeout — drops this
        // future, and dropping a future does not kill the process it spawned. Without this an
        // `apt install` keeps running against the same dpkg lock the rollback is about to take,
        // and whatever it completes is in no history that could compensate it.
        //
        // On Unix `Stopping` takes that job over, because tokio's version of it is SIGKILL and
        // **SIGKILL is not a way to stop a package manager** (see that type). Windows has no
        // gentler signal to send, so there it stays exactly as it was.
        #[cfg(windows)]
        command.kill_on_drop(true);
        #[cfg(unix)]
        {
            command.kill_on_drop(false);
            // Own process group: `Stopping` stops the tree, and a tree is only addressable if
            // the child leads its own group (`sudo` above, manager below, wrappers between).
            command.process_group(0);
        }

        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let interactive = self.stdin == ChildStdin::Interactive;
        // **No terminal for a mutation's child, and that is the whole of `#132`.** It leads its
        // own process group (above, so `Stopping` can signal the tree), so it is never the
        // terminal's foreground group — and a group that is not the foreground group is stopped
        // by the kernel at its first read (`SIGTTIN`) rather than told there is nothing there.
        // `Stdio::inherit()` was therefore a handle no manager could ever successfully read:
        // `apt`, `pip` and `pacman` each sat stopped at their own question until
        // `command_idle_timeout_secs` (900s) expired, and were then reported as commands that
        // had gone quiet. Closed, the same read is an EOF — a failure the manager reports in its
        // own words, immediately.
        //
        // Nothing is given up by it: `S88` puts `-n` on every escalated command, so `sudo` cannot
        // ask here either, and the one prompt Shall allows itself is `sudo -v` — spawned by
        // `ensure_sudo_credentials` in Shall's own group, where it is the foreground group and
        // can read. Windows has no process groups, so there `inherit` is a console the child
        // really can read.
        #[cfg(unix)]
        command.stdin(Stdio::null());
        #[cfg(windows)]
        command.stdin(if interactive && std::io::stdin().is_terminal() {
            Stdio::inherit()
        } else {
            Stdio::null()
        });

        let child = command.spawn().map_err(|e| {
            let message = format!("Failed to spawn {}: {}", cmd, e);
            // A program that is not on PATH does not arrive during a backoff.
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::command_failed_permanently(message)
            } else {
                Error::command_failed(message)
            }
        })?;

        // A mutation can run for minutes. Its progress used to reach the terminal because the
        // handles were inherited — which is exactly what emptied `output.stdout` and broke
        // every parser. Capture it and mirror it instead, so the bytes go both places.
        // The mirror is stderr, never stdout: stdout carries Shall's own answer, and a child's
        // chatter interleaved with it is not parseable by whoever piped us.
        let mirror = interactive && std::io::stderr().is_terminal();
        Self::wait_watched(child, &describe(cmd, args), mirror, self.idle).await
    }

    fn shares_stdin(&self) -> bool {
        self.stdin == ChildStdin::Interactive
    }

    fn check_command(&self, cmd: &str) -> bool {
        program_exists(cmd)
    }

    async fn symlink(&self, src: &Path, dst: &Path) -> Result<()> {
        #[cfg(unix)]
        {
            tokio::fs::symlink(src, dst)
                .await
                .map_err(|e| Error::Io(e.to_string()))
        }
        #[cfg(windows)]
        {
            if src.is_dir() {
                tokio::fs::symlink_dir(src, dst)
                    .await
                    .map_err(|e| Error::Io(e.to_string()))
            } else {
                tokio::fs::symlink_file(src, dst)
                    .await
                    .map_err(|e| Error::Io(e.to_string()))
            }
        }
    }
}
