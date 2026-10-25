pub use serde::{Deserialize, Serialize};
pub use shall::core::artifact_lock::*;
pub use shall::core::ledger::LockFile;
use std::collections::BTreeMap;
use tempfile::TempDir;

fn lock(asset: &str, sha: Option<&str>) -> ArtifactLock {
    ArtifactLock {
        version: Some("10.2.0".into()),
        asset: asset.into(),
        url: format!("https://example.invalid/{}", asset),
        format: "tarball".into(),
        selected_by: None,
        sha256: sha.map(str::to_string),
        ..Default::default()
    }
}
fn system_lock(asset: &str, installer: &str, pkg: &str) -> ArtifactLock {
    ArtifactLock {
        version: Some("10.2.0".into()),
        asset: asset.into(),
        url: format!("https://example.invalid/{}", asset),
        format: "deb".into(),
        installed_by: Some(installer.into()),
        system_package: Some(pkg.into()),
        ..Default::default()
    }
}
mod a_changed_hash_inside_a_set_is_still_an_objection;
mod a_changed_hash_on_the_same_asset_is_an_objection;
mod a_declaration_that_installed_several_locks_all_of_them;
mod a_different_asset_is_an_objection_that_says_how_to_accept_it;
mod a_handoff_lock_survives_a_toml_round_trip;
mod a_locked_hash_with_no_new_hash_is_an_objection;
mod a_missing_file_is_an_empty_ledger_not_an_error;
mod a_set_that_gained_a_file_is_an_objection_naming_what_was_locked;
mod a_set_that_lost_a_file_is_an_objection_naming_it;
mod an_entry_survives_a_round_trip;
mod an_unhashed_lock_still_checks_the_asset;
mod forgetting_a_package_drops_its_entry;
mod nothing_locked_is_an_empty_slice_not_a_missing_answer;
mod recording_replaces_the_whole_set;
mod system_packages_reports_only_handoff_locks;
mod the_same_asset_and_hash_is_no_objection;
mod the_same_set_in_another_order_is_no_objection;
