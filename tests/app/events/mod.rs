use serde_json::json;
pub use shall::app::events::*;
pub use shall::config::Config;
pub use shall::core::hook_lock::{event_id, hash_script, refusal, HookLedger};
pub use shall::core::{Error, LockFile, Result};
pub use shall::model::event::{Event, Payload};
pub use std::path::{Path, PathBuf};
pub use tracing::{debug, warn};

fn config_at(root: &Path) -> Config {
    Config {
        config_root: root.to_path_buf(),
        preferences_file: root.join("preferences.toml"),
        ..Default::default()
    }
}
fn write_repo_hook(root: &Path, event: Event, body: &str) {
    let dir = root.join("hooks");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(event.as_str()), body).unwrap();
}
fn touch_script(marker: &Path) -> String {
    let p = marker.display();
    if cfg!(windows) {
        format!("New-Item -ItemType File -Path '{}' | Out-Null\n", p)
    } else {
        format!("#!/bin/sh\ntouch '{}'\n", p)
    }
}
fn capture_stdin_script(out: &Path) -> String {
    let p = out.display();
    if cfg!(windows) {
        format!("[Console]::In.ReadToEnd() | Set-Content -Path '{}'\n", p)
    } else {
        format!("#!/bin/sh\ncat > '{}'\n", p)
    }
}
fn exit_with(code: i32) -> String {
    if cfg!(windows) {
        format!("exit {}\n", code)
    } else {
        format!("#!/bin/sh\nexit {}\n", code)
    }
}
mod a_failing_hook_does_not_fail_the_sync;
mod a_file_named_after_a_typo_is_not_loaded;
mod a_repo_hook_and_a_machine_hook_for_one_event_both_load;
mod an_after_sync_under_hooks_is_not_an_event_hook;
mod an_approved_hook_runs_and_is_told_what_happened;
mod an_empty_hook_file_is_not_a_hook;
mod an_unapproved_hook_does_not_run;
mod an_undeclared_event_has_no_hooks;
mod editing_a_hook_makes_it_unapproved_again;
mod editing_an_approved_hook_stops_it_running;
mod firing_an_undeclared_event_costs_nothing;
mod the_two_locations_have_different_ledger_identities;
mod unapproved_lists_what_lock_would_fix;
