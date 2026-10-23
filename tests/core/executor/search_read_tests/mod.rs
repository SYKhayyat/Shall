use dashmap::DashMap;
pub use shall::core::executor::{CommandExecutor, DryRunOutput, MockExecutor, StdOutput};
pub use std::path::PathBuf;
use std::sync::Arc;

fn exec(cmdline: &str, response: StdOutput) -> (CommandExecutor, Arc<MockExecutor>) {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_response(cmdline, Ok(response));
    let e = CommandExecutor::with_layer(false, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    (e, mock)
}
mod a_quiet_nonzero_exit_is_an_ordinary_empty_result;
mod a_search_that_could_not_run_is_an_error_not_an_empty_answer;
mod a_warning_alongside_a_successful_run_is_not_a_fault;
