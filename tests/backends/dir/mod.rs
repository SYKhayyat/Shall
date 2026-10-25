use async_trait::async_trait;
pub use shall::backends::dir::*;
pub use shall::core::account::Account;
pub use shall::core::{BackendCore, CommandExecutor, Error, Installable, PackageSpec, Result};
pub use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::tempdir;
use tracing::info;

fn spec(name: &str, options: &[(&str, &str)]) -> PackageSpec {
    let mut opts = shall::config::grammar::Options::default();
    for (key, value) in options {
        opts.set(*key, *value);
    }
    PackageSpec {
        name: name.to_string(),
        backend: "dir".into(),
        options: opts,
        requires: Vec::new(),
        present: true,
    }
}
fn installer() -> DirInstallable {
    let executor = CommandExecutor::new(false, false);
    let core = Arc::new(DirBackendCore::new(
        executor,
        Arc::new(shall::config::Config::default()),
    ));
    DirInstallable {
        core,
        created: Default::default(),
    }
}
mod a_dir_refuses_a_file_at_its_path;
mod a_directory_that_cannot_be_emptied_is_a_failure_and_not_a_removal;
mod a_home_relative_dir_is_still_resolved;
mod a_per_user_dir_is_created_and_converges_without_reapplying_ownership;
mod a_relative_dir_is_refused_at_key_construction_too;
mod a_relative_dir_is_refused_rather_than_read_against_the_working_directory;
mod an_absent_directory_is_a_successful_removal;
mod invalid_mode_is_refused_before_creating_a_directory;
mod the_creation_receipt_is_taken_once_and_does_not_outlive_its_run;
