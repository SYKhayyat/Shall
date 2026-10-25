use async_trait::async_trait;
use dashmap::DashMap;
use serde_json::Value;
pub use shall::backends::brew::*;
pub use shall::core::executor::MockExecutor;
pub use shall::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result, Searchable, Upgradable,
};
use std::sync::Arc;
use tracing::info;

fn mocked() -> (Arc<MockExecutor>, Arc<BrewBackendCore>) {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let exec =
        CommandExecutor::with_layer(false, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    (mock, Arc::new(BrewBackendCore::new(exec)))
}
fn brew_info_json(installed: &str) -> String {
    format!(
        r#"[{{"name":"jq","versions":{{"stable":"1.7.1"}},"cellar":"/opt/homebrew/Cellar","installed":{installed}}}]"#
    )
}
mod a_batch_of_formulae_is_one_brew_call;
mod brew_search_skips_section_headers;
mod every_brew_command_ends_its_options_before_the_name;
mod info_answers_installed_here_not_known_to_a_tap;
mod the_version_reported_is_the_one_on_disk;
