//! The credential state's own tests, in the folder `src/core/sudo.rs` owns.
//!
//! **The mirror is the rule.** `src/core/sudo.rs` owns `tests/core/sudo/`, and every test for
//! the credential state lives in a file of its own, named after the assertion it makes. The
//! helpers the assertions share stay here, in the module file, because a helper in `src/` could
//! not be reached from a test crate — `tests/` links the library without `cfg(test)`.
//!
//! **What these two can reach, and what they cannot.** They drive the public surface — an
//! executor, a mock layer, the calls the mock recorded — because that is the behaviour that
//! matters: a run asks once, and two runs are two answers. The memo's own cells are private, and
//! the test that reads them lives inside the module that owns them
//! (`src/core/sudo.rs`'s own `#[cfg(test)]` module), because a test crate links the library
//! without `cfg(test)` and cannot see a private field at all.

mod a_run_asks_for_credentials_once_and_its_clones_inherit_the_answer;
mod a_second_run_asks_for_its_own_credentials;

use dashmap::DashMap;
use shall::core::executor::{CommandExecutor, MockExecutor};
use std::path::PathBuf;
use std::sync::Arc;

/// An executor wired to a mock that records every call.
pub(crate) fn wired() -> (CommandExecutor, Arc<MockExecutor>) {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let executor =
        CommandExecutor::with_layer(false, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    (executor, mock)
}

/// How many times the run asked the machine whether it may escalate.
pub(crate) async fn probes(mock: &MockExecutor) -> usize {
    mock.get_calls()
        .await
        .iter()
        .filter(|c| c.as_str() == "sudo -n -v")
        .count()
}

/// Whether this host raises a credential probe at all.
///
/// **Asked of `escalates`, not assumed.** Windows never escalates and root does not need to, so
/// on those hosts there is no probe to count, and asserting one would be asserting a POSIX
/// behaviour on a machine that has none.
pub(crate) fn this_host_escalates() -> bool {
    CommandExecutor::escalates(true)
}
