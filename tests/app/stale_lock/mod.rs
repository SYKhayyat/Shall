pub use shall::app::stale_lock::*;
use std::collections::HashMap;
pub use std::path::{Path, PathBuf};

struct Fake {
    alive: Vec<u32>,
    running: Vec<&'static str>,
}
impl Processes for Fake {
    fn pid_alive(&self, pid: u32) -> bool {
        self.alive.contains(&pid)
    }
    fn any_named(&self, name: &str) -> bool {
        self.running.contains(&name)
    }
}
fn reader(files: &[(&str, &str)]) -> impl Fn(&Path) -> LockFile {
    let map: HashMap<String, String> = files
        .iter()
        .map(|(p, b)| (p.to_string(), b.to_string()))
        .collect();
    move |p: &Path| {
        map.get(&p.to_string_lossy().replace('\\', "/"))
            .map_or(LockFile::Absent, |b| LockFile::Body(b.clone()))
    }
}
fn unreadable(files: &[&str]) -> impl Fn(&Path) -> LockFile {
    let set: Vec<String> = files.iter().map(|p| p.to_string()).collect();
    move |p: &Path| {
        if set.contains(&p.to_string_lossy().replace('\\', "/")) {
            LockFile::Unreadable
        } else {
            LockFile::Absent
        }
    }
}
mod a_backend_belongs_to_at_most_one_lock;
mod a_backend_with_no_lock_is_never_waited_on;
mod a_backend_with_no_manager_lock_is_always_free;
mod a_cancelled_wait_stops_rather_than_serving_out_its_budget;
mod a_flock_is_held_only_while_something_is_running;
mod a_live_pid_holds_its_lock_even_with_no_process_of_that_name;
mod a_lock_a_live_manager_holds_says_to_wait;
mod a_lock_is_left_alone_while_its_manager_is_running;
mod a_lock_nobody_holds_is_not_waited_for;
mod a_lock_nothing_holds_says_it_is_stale_and_names_the_file;
mod a_lock_that_is_left_alone_is_still_reported;
mod a_lock_with_no_pid_is_stale_when_its_manager_is_not_running;
mod a_machine_with_no_locks_has_nothing_to_clear;
mod a_manager_that_is_running_but_holds_no_lock_file_holds_nothing;
mod a_pid_file_is_judged_by_its_own_pid;
mod a_row_that_may_not_be_cleared_carries_its_reason;
mod a_zombie_is_read_out_of_stat_whatever_the_process_was_called;
mod an_unreadable_pid_is_not_evidence_of_staleness;
mod an_unreadable_pid_lock_is_still_left_alone;
mod an_unreadable_pidless_lock_is_left_alone_while_its_manager_runs;
mod an_unreadable_pidless_lock_is_still_stale_when_nothing_holds_it;
mod each_managers_own_words_for_a_taken_lock_are_recognised;
mod every_row_names_a_manager_and_at_least_one_path;
mod held_for_calls_an_unreadable_pidless_lock_stale_rather_than_free;
mod no_flock_style_lock_is_ever_reported_as_clearable;
mod the_aur_helpers_are_answered_by_pacmans_lock;
