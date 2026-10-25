use dashmap::DashMap;
pub use shall::core::account::Account;
pub use shall::core::account_exec::*;
pub use shall::core::executor::{CommandExecutor, MockExecutor, ReadOutcome, StdOutput};
pub use shall::core::{Error, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

fn wired() -> (CommandExecutor, Arc<MockExecutor>) {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let e = CommandExecutor::with_layer(false, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    (e, mock)
}
fn another_account() -> Account {
    for name in ["nobody", "daemon", "bin", "root"] {
        if let Ok(account) = Account::resolve(name) {
            if !account.is_this_process() {
                return account;
            }
        }
    }
    panic!("this host names no account other than the invoking one");
}
fn account_env(account: &Account) -> HashMap<String, String> {
    account.session_env().into_iter().collect()
}
mod a_command_for_another_account_is_refused_where_this_process_cannot_become_it;
mod a_command_for_the_account_this_process_is_runs_unwrapped;
mod escalation_and_account_switching_compose_into_one_wrapper;
mod escalation_without_an_account_is_unchanged;
mod the_account_this_process_is_is_not_handed_to_the_composer_as_a_target;
mod the_variables_inside_the_wrapper_are_in_a_stable_order;
