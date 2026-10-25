use dashmap::DashMap;
use serde::Deserialize;
use serde_json::Value;
pub use shall::backends::generic::{
    CacheClean, DependsProbe, GenericBackendCore, GenericEnumerable, GenericInstallable,
    GenericQueryable, GenericRepoManager, GenericSearchable, GenericUpgradable, ManagerConfig,
    ManualListing, RepoListing, SearchSource, VersionPin,
};
pub use shall::backends::onboarder::*;
pub use shall::backends::BackendRegistry;
pub use shall::core::{BackendCapabilities, CommandExecutor, LockFile, Package};
pub use shall::parsers::OutputParser;
pub use shall::utils::text::sanitize;
use std::path::Path;
use std::sync::Arc;
pub use tracing::{error, warn};

fn firewall_def() -> CustomBackendDef {
    CustomBackendDef {
        name: "firewall".into(),
        binary: Some("ufw".into()),
        remove_binary: None,
        install_args: vec!["allow".into()],
        remove_args: vec!["delete".into(), "allow".into()],
        list_args: vec!["status".into()],
        ..Default::default()
    }
}
fn spec(backend: &str, name: &str) -> shall::core::PackageSpec {
    shall::core::PackageSpec {
        name: name.into(),
        backend: backend.into(),
        ..Default::default()
    }
}
fn mock_exec() -> (Arc<shall::core::executor::MockExecutor>, CommandExecutor) {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(shall::core::executor::MockExecutor::new(vfs.clone()));
    let exec =
        CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    (mock, exec)
}
fn write_repo(dir: &Path, body: &str) -> std::path::PathBuf {
    let file = dir.join("backends.toml");
    std::fs::write(&file, body).unwrap();
    file
}
const PARU_TOML: &str = r#"
[[backend]]
name = "paru"
install_args = ["-S"]
remove_args = ["-R"]
list_args = ["-Qm"]
"#;
mod a_binary_that_names_a_path_is_accepted;
mod a_custom_backend_is_a_first_class_peer_when_it_says_so;
mod a_definition_that_does_not_say_so_cannot_take_a_name;
mod a_delimiter_beats_the_quote_rule;
mod a_leading_tilde_expands_to_home;
mod a_name_that_differs_from_its_binary_runs_the_binary_on_every_verb;
mod a_name_the_prefix_grammar_already_spends_is_refused;
mod a_quoted_column_survives_the_space_inside_it;
mod a_repo_definition_registers_once_it_is_approved;
mod an_edited_definition_file_stops_registering_until_it_is_re_approved;
mod an_empty_or_spaced_binary_is_still_refused;
mod columns_parser_extracts_name_and_version;
mod columns_parser_skips_header_and_prefixes;
mod json_parser_array_of_objects_with_path;
mod json_parser_object_keys_as_names;
mod json_parser_supports_array_path_indices;
mod lines_parser_one_name_per_line;
mod no_definition_file_is_not_an_error;
mod overrides_replaces_the_definition_that_held_the_name;
mod regex_parser_named_captures;
mod registers_valid_and_skips_collisions_and_bad_names;
