use async_trait::async_trait;
use dashmap::DashMap;
pub use serde::{Deserialize, Serialize};
pub use shall::app::sync::guard::{GuardScope, Reaped};
pub use shall::backends::artifact::teardown::{still_installed, tear_down, Deployed};
pub use shall::backends::artifact::{system_pkg, Format};
pub use shall::backends::web::*;
pub use shall::core::executor::DryRunOutput;
pub use shall::core::executor::MockExecutor;
pub use shall::core::{
    security::verify_checksum, BackendCore, CommandExecutor, Error, Installable, MetadataProvider,
    Package, PackageSpec, Queryable, Result,
};
pub use shall::utils::archive::extract_archive;
use std::collections::HashMap;
pub use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
pub use tracing::{debug, info};

fn backend(tag: &str) -> (WebInstallable, Arc<MockExecutor>, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let vfs: Arc<DashMap<std::path::PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let exec =
        CommandExecutor::with_layer(false, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    let core = Arc::new(WebBackendCore::new(
        exec,
        tmp.path().join(tag),
        tmp.path().join("bin"),
        true,
        false,
        Vec::new(),
    ));
    (WebInstallable { core }, mock, tmp)
}
async fn record(core: &WebBackendCore, url: &str, entry: WebState) {
    core.commit_state(vec![(url.to_string(), entry)], Vec::new())
        .await
        .expect("writing the state");
}
fn handed_to(installer: &str, package: &str, url: &str) -> WebState {
    WebState {
        url: url.to_string(),
        // A handoff owns no tree of Shall's — the manager placed the files.
        local_path: String::new(),
        bin_link: None,
        etag: None,
        last_modified: None,
        installed_by: Some(installer.to_string()),
        system_package: Some(package.to_string()),
    }
}
async fn reaped() -> Reaped {
    crate::harness::reaped_for_a_test(GuardScope::Remove).await
}
fn removal_argv(line: &str) -> String {
    let mut parts = line.split(' ');
    let cmd = parts.next().unwrap_or_default();
    let args: Vec<&str> = parts.collect();
    CommandExecutor::as_launched(cmd, &args, true)
}
mod a_commit_beside_a_removal_keeps_both_answers;
mod a_failed_removal_keeps_the_record_rather_than_forgetting_the_resource;
mod a_resource_a_system_manager_owns_is_removed_through_that_manager;
mod an_installer_with_no_known_removal_is_reported_not_skipped;
mod removing_something_that_was_never_recorded_is_not_a_failure;
mod the_rpm_twin_removes_by_name_too;
mod two_concurrent_installs_do_not_lose_each_other;
