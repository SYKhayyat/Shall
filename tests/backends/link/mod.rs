use async_trait::async_trait;
pub use shall::backends::link::*;
pub use shall::config::Config;
pub use shall::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, PackageSpec, Result,
};
pub use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::tempdir;
pub use tera::{Context, Tera};
pub use tracing::{debug, info, warn};

fn installer() -> LinkInstallable {
    // A real (non-dry-run) executor so backup copies and writes hit the tempdir.
    let exec = CommandExecutor::new(false, false);
    let core = Arc::new(LinkBackendCore::new(exec, Arc::new(Config::default())));
    LinkInstallable { core }
}
fn installer_rooted_at(cfg: &Config) -> LinkInstallable {
    let exec = CommandExecutor::new(false, false);
    let core = Arc::new(LinkBackendCore::new(exec, Arc::new(cfg.clone())));
    LinkInstallable { core }
}
fn inline_spec(target: &Path, content: &str) -> PackageSpec {
    let mut options = shall::config::grammar::Options::default();
    options.set("target", target.to_string_lossy().to_string());
    options.set("content", content.to_string());
    PackageSpec {
        name: target.to_string_lossy().to_string(),
        backend: "link".into(),
        options,
        requires: vec![],
        present: true,
    }
}
fn decrypt_spec(source: &Path, target: &Path, tool: &str) -> PackageSpec {
    let mut options = shall::config::grammar::Options::default();
    options.set("target", target.to_string_lossy().to_string());
    options.set("decrypt", tool.to_string());
    PackageSpec {
        name: source.to_string_lossy().to_string(),
        backend: "link".into(),
        options,
        requires: vec![],
        present: true,
    }
}
async fn removal_token() -> shall::app::sync::guard::Reaped {
    crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Sync).await
}
mod a_bare_tilde_is_the_home_directory_and_is_inside_it;
mod a_content_link_names_no_source_and_is_not_asked_for_one;
mod a_destination_that_lands_in_the_home_is_not_outside_the_home;
mod a_directory_at_the_target_is_backed_up_before_the_link_replaces_it;
mod a_link_declared_the_way_the_readme_writes_it_can_actually_be_read;
mod a_link_to_a_file_that_is_not_there_is_refused_and_the_message_says_where_it_looked;
mod a_links_ledger_key_is_its_destination_not_its_source;
mod a_per_user_template_and_target_use_the_same_account_home;
mod a_relative_source_is_read_from_the_config_repo_whichever_declaration_names_it;
mod a_secret_is_refused_through_a_parent_that_symlinks_into_the_repo;
mod a_secret_is_restricted_by_the_time_it_exists;
mod a_secret_may_not_be_decrypted_into_the_config_repo;
mod a_secret_write_leaves_no_backup_beside_it;
mod a_system_path_is_outside_home;
mod a_template_is_rendered_with_the_facts_about_this_machine;
mod a_template_names_the_home_it_is_rendered_for;
mod a_template_needing_an_undetectable_home_fails_naming_it;
mod a_template_that_does_not_parse_is_an_error_and_writes_nothing;
mod a_users_edit_after_adoption_does_not_clobber_the_original_backup;
mod backs_up_preexisting_file_then_writes_managed_content;
mod backup_defaults_on_and_only_no_or_false_opts_out;
mod backup_no_opts_a_single_line_out_of_the_backup;
mod decrypt_dry_run_writes_nothing;
mod decrypt_unknown_tool_errors;
mod no_backup_created_when_target_absent;
mod plain_symlink_ownership_does_not_modify_its_source;
mod removing_a_declaration_restores_what_was_there_before;
mod removing_a_declaration_that_took_over_nothing_removes_the_file;
mod secrets_in_template_finds_all_secret_references;
mod secrets_in_template_handles_empty_and_edge_cases;
mod teardown_never_deletes_a_directory_shall_did_not_place;
mod teardown_restores_a_backed_up_directory_tree;
mod the_shared_context_carries_every_fact_install_and_check_agree_on;
mod the_source_path_goes_behind_the_terminator_and_the_identity_stays_in_front;
