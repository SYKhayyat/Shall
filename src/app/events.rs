//! Running the scripts attached to Shall's own events (XIII.13, U15).
//!
//! **Two locations, and both run (U15, ruled 2026-07-24).** A hook may live in the config
//! repo's `hooks/` directory — the policy every machine should run, committed and shared — or
//! in `preferences.toml`'s `[hooks]` table, which is where a machine keeps the notification
//! that talks to *its* Slack. The two kinds of hook are genuinely different, and forcing them
//! into one file makes one of them wrong.
//!
//! **Additive, never overriding.** Both fire, repo first. A precedence rule would mean that
//! adding a local notification silently disables the shared policy, which is the quiet failure
//! this whole model exists to avoid.
//!
//! **A failing hook warns; it does not fail the sync.** The sync's job is the machine's state,
//! and it succeeded or it did not — a Slack webhook that is down does not change that. The
//! inverse (a hook that can fail a converged sync) would make every integration a new way for
//! `sync` to break.
//!
//! **Approved before it runs, like everything else the repo can execute (II.12).** Same ledger,
//! same rule, no exception for "it's only a notification".

use crate::config::Config;
use crate::core::hook_lock::{event_id, hash_script, refusal, HookLedger};
use crate::core::LockFile;
use crate::core::{Error, Result};
use crate::model::event::{Event, Payload};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// One script attached to one event, and where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventHook {
    pub event: Event,
    /// The ledger identity — `event:on_drift@hooks/on_drift`. Distinct per location, so
    /// approving the repo's hook never silently approves this machine's.
    pub id: String,
    /// Where it came from, for a message a reader can act on.
    pub origin: String,
    pub script: String,
}

/// Every event hook this machine would run, from both locations.
#[derive(Debug, Clone, Default)]
pub struct EventHooks {
    hooks: Vec<EventHook>,
    locks_dir: PathBuf,
}

impl EventHooks {
    /// Collect the hooks from the config repo and from `preferences.toml`.
    ///
    /// Repo first, so the shared policy runs before the local notification — a hook that tells
    /// you what happened should be told after the hook that does something about it.
    pub fn load(config: &Config) -> EventHooks {
        let root = config.config_root();
        let mut hooks = Vec::new();
        for event in Event::ALL {
            if let Some(hook) = repo_hook(&root, event) {
                hooks.push(hook);
            }
            if let Some(hook) = preference_hook(config, event) {
                hooks.push(hook);
            }
        }
        EventHooks {
            hooks,
            locks_dir: config.layout().locks_dir(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }

    pub fn all(&self) -> &[EventHook] {
        &self.hooks
    }

    /// Fire an event: run every hook attached to it, in load order.
    ///
    /// **An undeclared event costs nothing** — no ledger read, no process, no allocation beyond
    /// the scan that found no hooks. That is what lets these be sprinkled at the interesting
    /// moments without asking whether anyone is listening.
    pub async fn fire(&self, event: Event, data: serde_json::Value) {
        let attached: Vec<&EventHook> = self.hooks.iter().filter(|h| h.event == event).collect();
        if attached.is_empty() {
            return;
        }
        let stdin = Payload::new(event, data).to_stdin();

        // Read the ledger once for the whole event, not once per hook.
        let ledger = match HookLedger::load(&HookLedger::path_in(&self.locks_dir)) {
            Ok(l) => l,
            // A ledger that cannot be read is not permission to run unapproved code.
            Err(e) => {
                warn!(
                    "not running the `{}` hooks: the approval ledger could not be read ({}).",
                    event, e
                );
                return;
            }
        };

        for hook in attached {
            let verdict = ledger.verdict(&hook.id, &hash_script(&hook.script));
            if !verdict.is_approved() {
                // A refusal here warns rather than failing, for the same reason a hook failure
                // does: the machine's state is already what it should be. It is loud, because
                // an unapproved hook is a hook that did not run.
                warn!("{}", refusal(&hook.id, &hook.origin, &verdict));
                continue;
            }
            if let Err(e) = run(hook, &stdin).await {
                warn!(
                    "the `{}` hook at {} failed: {}. The sync itself was unaffected.",
                    event, hook.origin, e
                );
            }
        }
    }

    /// The hooks that would NOT run because their current script is unapproved (II.12).
    ///
    /// Event hooks warn-and-skip rather than blocking a sync — which is right (a down webhook
    /// is not a reason to fail a converged machine) but means they are the one supply-chain
    /// item nothing surfaces until the moment it silently does nothing. `check` asks this so a
    /// hook you wrote and forgot to `shall lock` is a line in "what needs you", not a surprise
    /// the next time the machine drifts.
    ///
    /// A ledger that cannot be read counts everything as unapproved: unreadable is not
    /// permission to run.
    pub fn unapproved(&self) -> Vec<&EventHook> {
        let ledger = HookLedger::load(&HookLedger::path_in(&self.locks_dir)).unwrap_or_default();
        self.hooks
            .iter()
            .filter(|h| !ledger.verdict(&h.id, &hash_script(&h.script)).is_approved())
            .collect()
    }

    /// Approve every event hook at its current hash — what `shall lock` does. The only path
    /// that writes an approval, so approval stays a deliberate act.
    pub fn approve_all(&self) -> Result<usize> {
        if self.hooks.is_empty() {
            return Ok(0);
        }
        let path = HookLedger::path_in(&self.locks_dir);
        HookLedger::update(&path, |ledger| {
            for hook in &self.hooks {
                ledger.approve(&hook.id, &hash_script(&hook.script));
            }
            Ok(self.hooks.len())
        })
    }
}

/// The config repo's `hooks/<event>` — the policy every machine runs.
///
/// A plain script file rather than a TOML string: it is a script, it wants a shebang and an
/// editor's syntax highlighting, and one file is one hash, which is what the approval ledger
/// wants too.
fn repo_hook(root: &Path, event: Event) -> Option<EventHook> {
    let path = root.join("hooks").join(event.as_str());
    let script = std::fs::read_to_string(&path).ok()?;
    if script.trim().is_empty() {
        return None;
    }
    let origin = format!("hooks/{}", event);
    Some(EventHook {
        event,
        id: event_id(event.as_str(), &origin),
        origin,
        script,
    })
}

/// `preferences.toml`'s `[events]` table — this machine's own.
///
/// A table apart from `[hooks]`, which is the package-lifecycle hooks (`before_install`, …) the
/// embedded Lua/Rhai interpreter runs. Both once read `[hooks]`, so a `preferences.toml`
/// `after_sync` fired twice — once as Lua, once as a script here. Shall's own events are their
/// own table now, and the two can never name the same key.
fn preference_hook(config: &Config, event: Event) -> Option<EventHook> {
    let script = config.events.get(event.as_str())?;
    if script.trim().is_empty() {
        return None;
    }
    let origin = "preferences.toml".to_string();
    Some(EventHook {
        event,
        id: event_id(event.as_str(), &origin),
        origin,
        script: script.clone(),
    })
}

/// Run one hook with the payload on stdin.
///
/// The script is written to a temporary file and handed to the platform's interpreter — the
/// same one `exec:` uses, from the same place, so a hook is not a second thing that works on
/// Linux and quietly does nothing on Windows.
///
/// 0600 before it is run: the file holds whatever the hook's author wrote, in a world-readable
/// temp directory, and it exists for as long as the hook takes.
async fn run(hook: &EventHook, stdin: &str) -> Result<()> {
    use std::io::Write;

    debug!("firing {} from {}", hook.event, hook.origin);

    let body = hook.script.clone();
    let script = tokio::task::spawn_blocking(move || -> Result<tempfile::TempPath> {
        let mut tmp = tempfile::Builder::new()
            .suffix(crate::model::script::SCRIPT_SUFFIX)
            .tempfile()
            .map_err(Error::from)?;
        tmp.write_all(body.as_bytes()).map_err(Error::from)?;
        tmp.flush().map_err(Error::from)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(tmp.path())
                .map_err(Error::from)?
                .permissions();
            // 0600, matching the hook runner: an interpreter named on the command line reads
            // the file, so it never needs the execute bit — and this is the author's script,
            // sitting in a world-readable temp directory for as long as the event takes.
            perms.set_mode(0o600);
            std::fs::set_permissions(tmp.path(), perms).map_err(Error::from)?;
        }
        // The handle is closed here, keeping only the path (and the delete-on-drop). Windows
        // refuses to let a second process open a file this one still holds, so an interpreter
        // launched against a live `NamedTempFile` reads nothing and reports nothing useful.
        Ok(tmp.into_temp_path())
    })
    .await
    .map_err(|e| Error::Other(e.to_string()))??;

    let launch = crate::model::script::launch_for(&script, &hook.script)?;
    let mut command = tokio::process::Command::new(&launch.program);
    command
        .args(&launch.args)
        .env("SHALL_EVENT", hook.event.as_str())
        .env("SHALL_OS", std::env::consts::OS)
        .env("SHALL_ARCH", std::env::consts::ARCH);

    // Supervised: an event hook is arbitrary code fired by a timer or a package-manager hook,
    // with nobody at the terminal. Unowned and unbounded, one that waited on something waited
    // forever, and one abandoned by whatever fired it kept running.
    let out = crate::core::supervise::supervised_output_fed(command, "the hook", true, stdin)
        .await
        .map_err(|e| Error::Other(format!("could not start the hook: {}", e)))?;
    if !out.status.success() {
        return Err(Error::Other(format!("it exited {:?}", out.status.code())));
    }
    Ok(())
}
