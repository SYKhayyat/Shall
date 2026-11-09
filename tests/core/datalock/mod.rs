use fs2::FileExt;
pub use shall::core::datalock::*;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
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
/// A whole other process holding the lock, which is what the lock is for.
///
/// **The wait needs a holder that will actually let go.** A direct `acquire` in this process
/// still waits for a same-process holder, but a test that holds the guard in its own thread has
/// no way to release it from inside the wait — so the holder is a separate process, and the
/// contention the wait sees is the one the lock exists for. The child is this same test binary
/// re-invoked with `--exact`, so it runs the test below and nothing else.
const HOLD_FOR_THE_PARENT: &str = "SHALL_TEST_HOLD_THE_LOCK_IN";

/// The readiness marker the child leaves in the directory it holds.
///
/// **Not its stdout.** libtest writes a preamble of its own first, so a parent that read one
/// line and expected `HOLDING` read a blank line — and then closed the pipe, killing the child
/// that was supposed to be holding the lock. A file the child creates after the lock is taken
/// is the same fact with none of the buffering.
const HOLDING_MARKER: &str = "HOLDING";

fn child_test_name() -> String {
    // libtest names tests without the crate's own prefix: `suite::core::datalock` is
    // `core::datalock` to it.
    let path = module_path!();
    let (_, module) = path.split_once("::").unwrap_or(("", path));
    format!("{module}::a_child_process_holds_the_lock_for_its_parent")
}

/// The child half. In the parent's own run the switch is absent and this is a no-op.
#[test]
fn a_child_process_holds_the_lock_for_its_parent() {
    let Some(dir) = std::env::var_os(HOLD_FOR_THE_PARENT) else {
        return;
    };
    let path = PathBuf::from(dir);
    let held = DataLock::acquire(&path, "the child", Duration::from_secs(5))
        .expect("the child could not take a free directory");
    std::fs::write(path.join(HOLDING_MARKER), b"").expect("the readiness marker");
    std::thread::sleep(Duration::from_millis(1500));
    drop(held);
}

struct ChildHolder {
    child: Child,
}

impl ChildHolder {
    fn on(dir: &Path) -> Self {
        let marker = dir.join(HOLDING_MARKER);
        let _ = std::fs::remove_file(&marker);
        let child = Command::new(std::env::current_exe().expect("the test binary has a path"))
            .arg("--exact")
            .arg(child_test_name())
            .env(HOLD_FOR_THE_PARENT, dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the child test binary did not start");
        let deadline = Instant::now() + Duration::from_secs(10);
        while !marker.exists() {
            assert!(
                Instant::now() < deadline,
                "the child never reported holding the lock -- is `{}` still the test's name?",
                child_test_name()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        Self { child }
    }
}

impl Drop for ChildHolder {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
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
mod a_lock_on_one_directory_does_not_excuse_a_write_to_another;
mod a_lock_this_process_holds_is_not_taken_again;
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
mod the_owner_stamp_is_compared_to_this_process;
