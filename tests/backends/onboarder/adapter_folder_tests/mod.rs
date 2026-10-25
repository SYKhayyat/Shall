use serde::Deserialize;
use serde_json::Value;
pub use shall::backends::generic::{
    CacheClean, DependsProbe, GenericBackendCore, GenericEnumerable, GenericInstallable,
    GenericQueryable, GenericRepoManager, GenericSearchable, GenericUpgradable, ManagerConfig,
    ManualListing, RepoListing, SearchSource, VersionPin,
};
pub use shall::backends::onboarder::*;
pub use shall::backends::BackendRegistry;
pub use shall::core::hook_lock::{adapter_id, hash_script, HookLedger};
pub use shall::core::{BackendCapabilities, CommandExecutor, LockFile, Package};
pub use shall::parsers::OutputParser;
pub use shall::utils::text::sanitize;
use std::path::Path;
use std::sync::Arc;
pub use tracing::{error, warn};

const BACKENDS: &str =
    "[[backend]]\nname = \"paru\"\ninstall_args = [\"-S\"]\nlist_args = [\"-Qm\"]\n";
const SETTINGS: &str = "[[setting_store]]\nname = \"kde\"\ndetect = \"kwriteconfig6\"\nread = [\"kreadconfig6\"]\nwrite = [\"kwriteconfig6\"]\nreset = [\"kwriteconfig6\"]\n";
fn approve(locks: &Path, file: &str, body: &str) {
    let path = HookLedger::path_in(locks);
    let mut l = HookLedger::load(&path).unwrap();
    l.approve(&adapter_id(file), &hash_script(body));
    l.save(&path).unwrap();
}
mod a_definition_reports_updates_with_a_batch_verb_and_without_one;
mod a_definition_without_an_outdated_verb_says_it_cannot_be_asked;
mod a_machine_listing_without_its_own_parser_is_refused_not_guessed;
mod each_adapter_file_is_approved_on_its_own;
mod the_adapters_folder_is_inside_the_config_repo;
