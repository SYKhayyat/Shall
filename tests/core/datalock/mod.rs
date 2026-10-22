use fs2::FileExt;
pub use shall::core::datalock::*;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("shall-datalock-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}
static TEST_GATE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
fn gate() -> (
    std::sync::MutexGuard<'static, ()>,
    tokio::sync::MutexGuard<'static, ()>,
) {
    let env = shall::core::shall_data_dir_lock();
    (env, TEST_GATE.blocking_lock())
}
fn lock_file_is_in(dir: &Path) -> bool {
    dir.join("shall.lock").exists()
}
struct DataDir(Option<std::ffi::OsString>);

impl DataDir {
    fn at(dir: &Path) -> Self {
        std::fs::create_dir_all(dir).unwrap();
        let previous = std::env::var_os("SHALL_DATA_DIR");
        std::env::set_var("SHALL_DATA_DIR", dir);
        Self(previous)
    }
}
impl Drop for DataDir {
    fn drop(&mut self) {
        match self.0.take() {
            Some(v) => std::env::set_var("SHALL_DATA_DIR", v),
            None => std::env::remove_var("SHALL_DATA_DIR"),
        }
    }
}
mod a_contended_lock_is_told_from_a_failure_that_is_not_one;
mod a_contended_lock_is_waited_for_and_then_taken;
mod a_deferred_step_takes_the_lock_when_it_is_free;
mod a_free_directory_hands_out_the_lock_rather_than_reporting_contention;
mod a_generation_file_that_cannot_be_read_is_unknown_not_zero;
mod a_hook_with_nobody_holding_the_lock_proceeds_rather_than_standing_down;
mod a_lock_is_taken_and_released_by_drop;
mod a_process_inside_the_lock_does_not_take_it_again;
mod a_second_holder_is_refused_with_who_holds_it_rather_than_hanging;
mod a_torn_generation_is_unknown_and_a_missing_one_is_zero;
mod a_wait_that_runs_out_first_spends_the_time_it_was_given;
mod a_write_from_outside_the_lock_takes_one;
mod a_writer_that_finishes_moves_the_generation;
mod an_owner_file_with_no_name_in_it_falls_back_rather_than_naming_nobody;
mod both_doors_read_a_busy_lock_the_same_way;
mod held_is_false_outside_the_lock_and_true_inside;
mod the_lock_file_names_its_holder;
mod the_off_runtime_wait_returns_holding_the_lock;
