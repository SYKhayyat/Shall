use dashmap::DashMap;
pub use shall::app::apply::prereq::*;
pub use shall::core::executor::{DryRunOutput, MockExecutor};
pub use shall::core::Result;
pub use shall::model::prereq::PrereqDef;
use std::path::PathBuf;
use std::sync::Arc;
pub use tracing::{info, warn};

struct Harness {
    exec: shall::core::CommandExecutor,
    mock: Arc<MockExecutor>,
    config: Arc<shall::config::Config>,
    registry: Arc<shall::backends::BackendRegistry>,
}
fn harness(answers: &[(&str, bool, &str)]) -> Harness {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    for (cmd, ok, out) in answers {
        let response = if *ok {
            DryRunOutput {
                stdout: out.as_bytes().to_vec(),
                stderr: vec![],
            }
            .into()
        } else {
            DryRunOutput::faulted(out)
        };
        mock.set_response(cmd, Ok(response));
    }
    let exec = shall::core::CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        Arc::new(DashMap::new()),
    );
    Harness {
        exec,
        mock,
        config: Arc::new(shall::config::Config::default()),
        registry: Arc::new(shall::backends::BackendRegistry::new()),
    }
}
impl Harness {
    fn prereqs(&self) -> Prereqs<'_> {
        Prereqs {
            config: &self.config,
            executor: &self.exec,
            registry: &self.registry,
        }
    }
    async fn ran(&self, cmd: &str) -> bool {
        self.mock.get_calls().await.iter().any(|c| c == cmd)
    }
}
fn builtin(manager: &str) -> PrereqDef {
    let file: shall::model::prereq::PrereqFile = toml::from_str(BUILTIN).unwrap();
    file.prereq
        .into_iter()
        .find(|r| r.manager == manager)
        .expect("a shipped row")
}
mod a_dry_run_sets_nothing_up;
mod a_per_package_row_asks_once_per_missing_package_only;
mod a_prerequisite_that_is_already_met_runs_nothing;
mod an_unmet_prerequisite_is_set_up_under_yes;
mod nothing_is_set_up_without_being_asked;
mod the_shipped_rows_are_available_without_a_repo_file;
