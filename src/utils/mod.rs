pub mod archive;
pub mod file;
pub mod progress;
pub mod regex_cache;
pub mod style;
pub mod text;

use std::path::PathBuf;

pub use archive::extract_archive;
pub use file::{
    bin_destination, deploy_executable, ensure_deployable, ensure_dir, read_lines_filtered,
    remove_deployed_path, strip_archive_suffixes,
};
pub use progress::{create_progress_reporter, ProgressHandle, ProgressReporter};

pub fn safe_data_dir() -> PathBuf {
    // `SHALL_DATA_DIR` overrides the OS data dir outright (used as-is, no `shall` suffix). This
    // lets a test harness or CI run against a throwaway, isolated state registry so it never
    // touches — or accumulates in — the user's real global state, and so a system-global
    // `prune` only ever sees the packages that run installed.
    //
    // **Present-but-empty is not "set".** The setter side filters empty values; reading one
    // unfiltered relocated the data root to the current directory, where `shall.lock` and
    // friends materialized mid-repo. Same rule both sides.
    if let Some(dir) = std::env::var_os("SHALL_DATA_DIR").filter(|v| !v.is_empty()) {
        return PathBuf::from(dir);
    }
    dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("shall")
}

pub fn safe_config_dir() -> PathBuf {
    // `SHALL_CONFIG_DIR` overrides the OS config dir outright (see `safe_data_dir`), and
    // **present-but-empty is not "set"** — the rule `safe_data_dir` states and this function
    // did not apply. `SHALL_CONFIG_DIR=` resolved the repo to the empty path, so `shall path`
    // printed a blank line and every write landed relative to the working directory.
    if let Some(dir) = std::env::var_os("SHALL_CONFIG_DIR").filter(|v| !v.is_empty()) {
        return PathBuf::from(dir);
    }
    dirs::config_dir()
        .unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(".config")
        })
        .join("shall")
}

/// Which variable decided the platform config directory, for `shall path --explain`.
///
/// `dirs::config_dir()` is the authority on the *path*; this names the input behind it,
/// because "the built-in default" withholds the one fact a reader needs when the directory is
/// not where they expected — which variable to change. `$XDG_CONFIG_HOME` outranks
/// `$HOME/.config` on Linux, so `HOME=` alone moves nothing (`#219`), and the answer sat in a
/// test comment before it was anywhere a user would look.
///
/// A relative or empty `$XDG_CONFIG_HOME` is ignored, which is the XDG rule and what `dirs`
/// implements — so this function follows the same precedence rather than inventing a second.
/// `the_explanation_names_the_variable_that_decided_it` ties the two together against the real
/// binary, so a change in `dirs` cannot leave this reporting a variable that decided nothing.
pub fn config_dir_source() -> &'static str {
    config_dir_source_from(&|name| std::env::var_os(name))
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn config_dir_source_from(get: &dyn Fn(&str) -> Option<std::ffi::OsString>) -> &'static str {
    if get("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .is_some_and(|p| p.is_absolute())
    {
        return "$XDG_CONFIG_HOME";
    }
    if get("HOME").filter(|v| !v.is_empty()).is_some() {
        return "$HOME/.config";
    }
    "the platform config directory"
}

#[cfg(target_os = "macos")]
pub fn config_dir_source_from(_get: &dyn Fn(&str) -> Option<std::ffi::OsString>) -> &'static str {
    "$HOME/Library/Application Support"
}

#[cfg(windows)]
pub fn config_dir_source_from(_get: &dyn Fn(&str) -> Option<std::ffi::OsString>) -> &'static str {
    "%APPDATA%"
}

// `refresh_path` was here, with a doc comment asserting that "backends that install a toolchain
// (mise, cargo) must call this before running hooks that invoke it". **Nothing called it, and
// there is no `cargo.rs`, since-deleted** — so either the bug it describes is live and this
// never guarded it,
// or the requirement is fiction. Deleting it makes that an open question instead of a solved
// one: a helper nobody calls, documented as mandatory, reads as coverage from every angle
// except the only one that counts.
//
// The live PATH mechanism is `executor::forget_path_lookups`, which drops the memo after
// Shall installs a manager — a different problem, and one with callers.
