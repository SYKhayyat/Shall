use async_trait::async_trait;
use serde_json::json;
pub use shall::backends::vscode::*;
pub use shall::core::{
    BackendCore, CommandExecutor, Installable, MetadataProvider, Package, PackageSpec, Queryable,
    RateLimiter, Result, Searchable, Upgradable,
};
use std::sync::Arc;
use tracing::info;

fn vscode_with(
    list_output: &str,
) -> (
    Arc<VscodeBackendCore>,
    Arc<shall::core::executor::MockExecutor>,
) {
    let vfs = Arc::new(dashmap::DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    mock.set_response(
        "code --list-extensions --show-versions",
        Ok(shall::core::executor::DryRunOutput {
            stdout: list_output.as_bytes().to_vec(),
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
    (Arc::new(VscodeBackendCore::new(exec)), mock)
}
mod a_batch_of_extensions_is_one_code_process;
mod an_extension_id_is_an_option_value_so_no_terminator_is_emitted;
mod info_answers_installed_here_not_published_to_the_marketplace;
