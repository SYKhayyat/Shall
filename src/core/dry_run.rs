//! `--dry-run` as a property of the run, not a habit of each verb.
//!
//! The flag was consulted per verb for as long as it has existed, and the result is the shape
//! this module deletes: `uninstall` checked it, `unmanage` checked it, `module create` checked
//! it — and `activate`, `deactivate`, `lock`, `git init` and `config init` did not. A preview
//! of "what happens if I switch to Work" left you switched to Work and printed nothing. That is
//! not five mistakes; it is one rule ("remember to check the flag") enforced by nothing, which
//! is the same finding as the guard that covered nine removal paths out of eleven.
//!
//! So the check moves to where the *write* happens. A verb added tomorrow inherits it by
//! calling the writer everything else calls, rather than by remembering a convention.
//!
//! **Why a process-wide value rather than a parameter.** `--dry-run` is a top-level flag parsed
//! once, before any command runs, and it applies to the whole process — there is no run in
//! which one write is a preview and another is not. Threading it to every write site would be
//! the per-verb habit again with a longer signature: a new call site would still have to be
//! given the flag by hand, and forgetting is exactly the failure being fixed. It is set once
//! from `main`, never from library code, and the setter is idempotent for a given run.

use std::sync::atomic::{AtomicBool, Ordering};

static DRY_RUN: AtomicBool = AtomicBool::new(false);

/// The marker every simulated action carries.
///
/// **It was typed by hand at sixty-eight call sites**, which is this module's own opening
/// paragraph one level down: one rule ("remember the marker, spelled exactly so") enforced by
/// nothing. It had already drifted four ways — `Would` beside `would`, `Go: [DRY-RUN]` with the
/// marker in second place so a `^\[DRY-RUN\]` grep misses the line entirely, and two sites on
/// `debug!`, which is **below the default log level**: `snapshot.rs` retention pruning and
/// `git.rs` committing both announced themselves to nobody. A preview that silently omits work
/// it would do is the preview failing at the one thing it is for.
///
/// `tests/review/dry_run_marker_tests.rs` is why it cannot drift again: the literal appears in this
/// file and nowhere else in `src/`.
pub const MARKER: &str = "[DRY-RUN]";

/// The tracing target every preview line carries, so the verbosity it needs is one directive
/// rather than a decision made at each call site.
///
/// **`info!` is not the default level.** This module asserted that it was, and the default is
/// `warn` (`entrypoint::log_level_from_argv`): so all 46 `would!` sites printed nothing unless
/// `-v` was passed, and `shall --dry-run deactivate X` answered a request for a preview with
/// complete silence (#153). The level is right — a preview is information, not a warning — so the
/// *filter* keeps this target visible instead of every call site escalating to `warn` and making
/// the vocabulary say "warning" about work nobody did.
pub const TARGET: &str = "shall::dry_run";

/// The filter directives for this run's verbosity, with previews kept visible.
///
/// `--quiet` is honoured: a user who asked for error-only output does not get a preview printed
/// under it. Every other level includes this target, so the marker appears at the verbosity the
/// run was invoked with and at higher ones.
pub fn with_previews_visible(level: &str) -> String {
    if level == "error" {
        level.to_string()
    } else {
        format!("{},{}={}", level, TARGET, "info")
    }
}

/// Report an action a preview did not take. Visible at the default verbosity, because the line
/// *is* the answer a `--dry-run` was asked for.
#[macro_export]
macro_rules! would {
    ($($arg:tt)*) => {
        tracing::info!(target: $crate::core::dry_run::TARGET, "{} {}", $crate::core::dry_run::MARKER, format_args!($($arg)*))
    };
}

/// As [`would!`], for a preview line that is *also* a warning about the machine — drift the run
/// found rather than work it declined to do. Reserved for that: everything in a preview is
/// undone, so "we did not do it" is not by itself a warning.
#[macro_export]
macro_rules! would_warn {
    ($($arg:tt)*) => {
        tracing::warn!(target: $crate::core::dry_run::TARGET, "{} {}", $crate::core::dry_run::MARKER, format_args!($($arg)*))
    };
}

/// As [`would!`], on stdout, for the verbs whose printed output *is* the answer to the
/// question the user asked rather than a log about answering it.
#[macro_export]
macro_rules! would_print {
    () => { println!("{}", $crate::core::dry_run::MARKER) };
    ($($arg:tt)*) => {
        println!("{} {}", $crate::core::dry_run::MARKER, format_args!($($arg)*))
    };
}

/// Record this process's `--dry-run` mode. Called once, from `main`, before dispatch.
pub fn set(on: bool) {
    DRY_RUN.store(on, Ordering::SeqCst);
}

/// Is this run a preview?
pub fn active() -> bool {
    DRY_RUN.load(Ordering::SeqCst)
}
