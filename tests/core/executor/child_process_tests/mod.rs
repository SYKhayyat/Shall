use dashmap::DashMap;
pub use shall::core::executor::*;
pub use shall::core::executor::{
    ChildStdin, CommandExecutor, ExecutionLayer, MockExecutor, RawExecutor,
};
pub use shall::core::Retryability;
use std::collections::HashMap;
pub use std::path::PathBuf;
use std::sync::Arc;

fn wired() -> (CommandExecutor, Arc<MockExecutor>) {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let e = CommandExecutor::with_layer(false, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    (e, mock)
}
fn tmpdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("shall-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}
fn silent_forever() -> (&'static str, Vec<String>) {
    #[cfg(unix)]
    {
        ("sleep", vec!["600".to_string()])
    }
    #[cfg(windows)]
    {
        (
            "powershell",
            vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                "Start-Sleep -Seconds 600".to_string(),
            ],
        )
    }
}
fn detaches_holding_stdout(seconds: usize) -> (String, Vec<String>) {
    #[cfg(unix)]
    {
        (
            "sh".to_string(),
            vec!["-c".to_string(), format!("sleep {seconds} & exit 0")],
        )
    }
    #[cfg(windows)]
    {
        let path = std::env::temp_dir().join(format!("shall-detach-{}.cmd", std::process::id()));
        std::fs::write(
            &path,
            format!(
                "@echo off\r\nstart /b cmd /c \"ping -n {} 127.0.0.1 > nul\"\r\nexit /b 0\r\n",
                seconds + 1
            ),
        )
        .expect("write the detaching fixture");
        (
            "cmd".to_string(),
            vec!["/C".to_string(), path.display().to_string()],
        )
    }
}
fn chatty_for_a_while(ticks: usize) -> (&'static str, Vec<String>) {
    #[cfg(unix)]
    {
        (
            "sh",
            vec![
                "-c".to_string(),
                format!("i=0; while [ $i -lt {ticks} ]; do echo tick; i=$((i+1)); sleep 0.5; done"),
            ],
        )
    }
    #[cfg(windows)]
    {
        (
            "powershell",
            vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                // `[Console]::Out`, not `Write-Output`: PowerShell buffers the pipeline
                // when stdout is redirected, so a fixture that "talks" through the
                // pipeline can be silent for its whole run — which is the one thing this
                // fixture must not be.
                format!(
                    "1..{ticks} | ForEach-Object {{ [Console]::Out.WriteLine('tick'); \
                     [Console]::Out.Flush(); Start-Sleep -Milliseconds 500 }}"
                ),
            ],
        )
    }
}
async fn interpreter_start_up_cost() -> std::time::Duration {
    let layer = RawExecutor::with_idle(ChildStdin::Closed, None);
    let (cmd, args) = chatty_for_a_while(1);
    let started = std::time::Instant::now();
    let _ = layer.execute(cmd, &args, &HashMap::new()).await;
    started.elapsed()
}
mod a_child_that_goes_silent_is_killed_and_named;
mod a_child_that_keeps_talking_outlives_the_bound;
mod a_detached_grandchild_cannot_hold_the_read_open_past_the_bound;
mod a_fabricated_status_carries_the_code_without_spawning_anything;
mod a_lock_directory_that_went_missing_is_made_again;
mod a_lock_key_cannot_escape_the_lock_directory;
mod a_read_carries_it_too;
mod a_read_is_bounded_tighter_than_a_mutation;
mod a_read_that_failed_but_explained_itself_is_left_to_its_caller;
mod a_read_that_failed_but_still_answered_keeps_its_answer;
mod a_read_that_failed_without_a_word_is_not_an_empty_machine;
mod a_read_whose_only_signal_is_its_exit_code_is_still_classified_and_retried;
mod a_real_executor_wires_a_reader_and_a_mutator;
mod an_unclassified_silent_failure_is_not_retried;
mod dropping_the_keepalive_guard_stops_the_loop;
mod every_spawn_carries_the_pager_suppression;
mod no_bound_means_no_bound;
mod opening_an_existing_lock_keeps_its_contents;
mod taking_the_exec_lock_does_not_truncate_a_planted_symlink;
mod the_killed_child_is_actually_dead;
mod the_reader_layer_never_shares_stdin;
mod the_suppression_overrides_a_user_pager;
