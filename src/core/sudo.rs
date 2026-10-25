//! What a run has established about escalating.
//!
//! **This is run state, and the run is what escalates.** One `App` builds one
//! `CommandExecutor`, every backend of that `App` holds a clone of it, and the clones share
//! what this module holds — so a run asks the machine for credentials once, and remembers the
//! answer for as long as the run lasts.
//!
//! **It was process-wide, and that was a bug rather than a preference.** Three `static`s
//! answered "has the credential probe been asked?" for the whole program, which meant that every
//! test in a binary shared one answer. A test that wanted to *watch* the probe was then at the
//! mercy of whichever test in the binary had asked first: the web backend's test failed on
//! nothing but libtest's thread order, and took the mutation gate's baseline down with it
//! (`#121`). Per run is both the scope the behaviour wanted and the scope the tests needed.

use crate::core::error::Error;

/// What this run has established about escalating.
///
/// The three cells are read, then acted on, and nothing joins them — so each carries the rule
/// that governs it, and the rules are the history of what went wrong.
pub struct SudoState {
    /// Whether this run has already proved it can escalate, so the probe runs once per run
    /// rather than once per command.
    ///
    /// The keepalive refreshes the timestamp every 60s for as long as a sync holds it, so a run
    /// that primes once stays primed. A false here costs one `sudo -n -v`, which is a few
    /// milliseconds; a true that is wrong costs nothing either, because the command itself still
    /// carries `-n` and fails by name rather than blocking.
    primed: std::sync::atomic::AtomicBool,

    /// Why this run cannot escalate, once it has found out.
    ///
    /// **Success was remembered and failure was not, and that asymmetry is the whole bug.** A
    /// refusal is as permanent as a success within one run — a wrong password does not become
    /// right during the same command, and a terminal with nobody at it does not grow a person —
    /// but `ensure_sudo_credentials` re-asked from scratch for every escalated invocation. The
    /// verbs that keep going after one backend fails are exactly the ones that pay for it:
    /// `update` refreshes each manager and "does not let one stop the rest", so it spent the
    /// full password bound *per manager*. Measured in the `tools` nightly, twice a night for
    /// weeks:
    ///
    /// ```text
    /// FAIL  sudo: a wrong password left Shall waiting 900s instead of reporting a failure
    /// FAIL  sudo: a terminal with nobody at it wedged Shall for 900s
    /// ```
    ///
    /// Both assertions bound the wait at 120s and both saw 900. The 120-second bound was working
    /// perfectly, once per backend.
    ///
    /// Not an `AtomicBool`: the *reason* is what the second caller has to report, and
    /// re-deriving it would mean asking sudo again, which is the thing being avoided.
    refused: std::sync::Mutex<Option<String>>,

    /// One task at a time may probe and prompt for sudo.
    ///
    /// **`primed` and `refused` are read, then acted on, and nothing joined the two.** *N*
    /// escalated commands start together in the first wave — ordinary with `max_parallel > 1`
    /// and more than one root-needing backend — and all *N* read `primed == false`, all *N* find
    /// no recorded refusal, all *N* run `sudo -n -v` and fail, and all *N* reach `sudo -v` with
    /// **inherited stdin**. That is several processes reading a password from one tty: keystrokes
    /// split between them, prompts interleave, and whichever fails first records a *permanent*
    /// refusal that then fails the whole run for the others.
    ///
    /// `S88` and `S89` are what made this reachable, by doing the right thing: `-n` on every
    /// command so no manager invocation sits on a prompt, and a remembered refusal so the bound
    /// costs 120s rather than 900. Both turned the priming call into the single funnel every
    /// escalated command passes through — which is exactly what turns an unsynchronised
    /// check-then-act into a thundering herd.
    ///
    /// One task probes and prompts; the rest wait for its answer and then re-read the two cells.
    priming: tokio::sync::Mutex<()>,
}

impl SudoState {
    /// **`pub` for the integration tests' sake.** The mirror links the library without
    /// `cfg(test)` and must construct the same run-scoped state.
    #[doc(hidden)]
    pub fn new() -> Self {
        Self {
            primed: std::sync::atomic::AtomicBool::new(false),
            refused: std::sync::Mutex::new(None),
            priming: tokio::sync::Mutex::new(()),
        }
    }

    pub(crate) fn is_primed(&self) -> bool {
        self.primed.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub(crate) fn mark_primed(&self) {
        self.primed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// The reason this run cannot escalate, if it has found one.
    /// **`pub` for the integration tests' sake.** The mirror links the library without
    /// `cfg(test)` and must inspect the remembered refusal.
    #[doc(hidden)]
    pub fn refusal(&self) -> Option<String> {
        self.refused.lock().ok().and_then(|s| s.clone())
    }

    /// The lock one task at a time may hold while it probes and prompts.
    ///
    /// Held across the probe, so the second task waits for the first one's answer and then
    /// re-reads the two cells rather than asking the machine again.
    pub(crate) async fn priming(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.priming.lock().await
    }

    /// Remember a refusal, and hand back the error to return.
    ///
    /// **The first reason is the one kept.** A later caller's guess must not overwrite the reason
    /// the first one established: the *first* refusal is the one that happened, and everything
    /// after it is this memo.
    /// **`pub` for the integration tests' sake.** The mirror links the library without
    /// `cfg(test)` and must drive the refusal memo directly.
    #[doc(hidden)]
    pub fn refuse(&self, why: String) -> Error {
        if let Ok(mut slot) = self.refused.lock() {
            if slot.is_none() {
                *slot = Some(why.clone());
            }
        }
        Error::command_failed_permanently(why)
    }
}
