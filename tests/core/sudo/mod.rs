use dashmap::DashMap;
pub use shall::core::error::Error;
use shall::core::executor::{CommandExecutor, MockExecutor};
pub use shall::core::sudo::SudoState;
pub use shall::core::sudo::*;
use std::path::PathBuf;
use std::sync::Arc;

pub(crate) fn wired() -> (CommandExecutor, Arc<MockExecutor>) {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let executor =
        CommandExecutor::with_layer(false, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    (executor, mock)
}

pub(crate) async fn probes(mock: &MockExecutor) -> usize {
    mock.get_calls()
        .await
        .iter()
        .filter(|call| call.as_str() == "sudo -n -v")
        .count()
}

pub(crate) fn this_host_escalates() -> bool {
    CommandExecutor::escalates(true)
}

mod a_run_asks_for_credentials_once_and_its_clones_inherit_the_answer;
mod a_second_run_asks_for_its_own_credentials;
mod a_sudo_refusal_is_remembered_and_the_first_reason_is_the_one_kept;
