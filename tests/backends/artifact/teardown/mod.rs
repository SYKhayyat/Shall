pub use shall::backends::artifact::system_pkg;
pub use shall::backends::artifact::teardown::*;
pub use shall::core::executor::CommandExecutor;
use std::path::PathBuf;

fn exec() -> CommandExecutor {
    CommandExecutor::new(true, false)
}
mod a_download_is_named_in_the_cache_by_its_last_url_segment;
mod a_record_with_nothing_deployed_removes_cleanly;
mod an_artifact_nobody_owns_hands_nothing_back;
mod an_empty_path_is_not_a_path;
mod the_failure_sentence_says_still_installed;
