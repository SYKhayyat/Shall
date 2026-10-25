use dashmap::DashMap;
pub use shall::app::reachable::*;
pub use shall::config::Config;
pub use shall::core::executor::{DryRunOutput, MockExecutor};
pub use shall::core::CommandExecutor;
pub use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(windows)]
const SANDBOX: &str = r"C:\shall-test-sandbox";
#[cfg(not(windows))]
const SANDBOX: &str = "/shall-test-sandbox";
fn mocked(answers: &[(&str, &str)]) -> (CommandExecutor, Config) {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    for (cmd, out) in answers {
        mock.set_response(
            cmd,
            Ok(DryRunOutput {
                stdout: out.as_bytes().to_vec(),
                stderr: vec![],
            }
            .into()),
        );
    }
    let exec = CommandExecutor::with_layer(false, false, mock, vfs, Arc::new(DashMap::new()));
    (exec, Config::sandboxed(Path::new(SANDBOX)))
}
mod a_per_user_ecosystem_names_its_bin_dir;
mod a_reachable_directory_is_not_warned_about;
mod a_system_manager_has_nothing_to_warn_about;
mod a_tool_that_cannot_answer_is_not_guessed_at;
mod a_warning_line_is_not_a_directory;
mod every_per_user_ecosystem_answers;
mod npms_prefix_offers_both_readings;
mod opam_answers_for_the_switch_that_is_selected;
mod the_directory_shall_itself_deploys_into_answers;
mod the_node_family_answers_from_the_tool;
mod the_warning_names_the_directory_and_the_line_to_add;
