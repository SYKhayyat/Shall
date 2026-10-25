use async_trait::async_trait;
use serde_json::Value;
pub use shall::backends::mise::*;
pub use shall::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result, Searchable, Upgradable,
};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

const REAL_LIST_JSON: &str = r#"{
  "jq": [
    {
      "version": "1.8.2",
      "requested_version": "latest",
      "install_path": "/root/.local/share/mise/installs/jq/1.8.2",
      "source": { "type": "mise.toml", "path": "/root/.config/mise/config.toml" },
      "installed": false,
      "active": false
    }
  ],
  "node": [
    {
      "version": "22.1.0",
      "requested_version": "latest",
      "install_path": "/root/.local/share/mise/installs/node/22.1.0",
      "source": { "type": "mise.toml", "path": "/root/.config/mise/config.toml" },
      "installed": true,
      "active": true
    }
  ]
}"#;
fn mise_with(
    list_json: &str,
) -> (
    Arc<MiseBackendCore>,
    Arc<shall::core::executor::MockExecutor>,
) {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    mock.set_response(
        "mise list --json",
        Ok(shall::core::executor::DryRunOutput {
            stdout: list_json.as_bytes().to_vec(),
            stderr: vec![],
        }
        .into()),
    );
    let exec = CommandExecutor::with_layer(
        false,
        false,
        mock.clone(),
        vfs,
        Arc::new(dashmap::DashMap::new()),
    );
    (Arc::new(MiseBackendCore::new(exec)), mock)
}
mod a_batch_of_tools_is_one_mise_call;
mod a_tool_mise_says_is_not_installed_is_not_listed;
mod an_empty_mise_lists_nothing;
mod info_answers_installed_here_not_known_to_mise;
mod mise_registry_filters_by_query;
mod mise_tool_names_come_after_the_terminator;
mod user_level_is_decided_by_where_the_source_lives;
