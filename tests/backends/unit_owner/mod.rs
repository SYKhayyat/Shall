//! `#211`: the row table that answers "which package owns this file", and the pass that asks it.

pub use shall::backends::unit_owner::*;
pub use shall::core::executor::{CommandExecutor, MockExecutor};
pub use std::sync::Arc;

use dashmap::DashMap;

/// A mock executor dressed as the tree's real one: the same layer stack every backend test uses,
/// so a command registered here answers exactly as a manager's would.
pub fn executor(mock: &Arc<MockExecutor>) -> CommandExecutor {
    CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        Arc::new(DashMap::new()),
        Arc::new(DashMap::new()),
    )
}

mod the_first_manager_that_can_answer_is_the_one_asked;
mod the_shipped_rows_parse_and_every_reader_answers;
