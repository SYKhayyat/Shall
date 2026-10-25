use async_trait::async_trait;
pub use serde::{Deserialize, Serialize};
pub use shall::app::sync::guard::{GuardScope, Reaped};
pub use shall::backends::appimage::*;
pub use shall::backends::artifact::teardown::{still_installed, tear_down, Deployed};
pub use shall::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result,
};
use std::collections::HashMap;
pub use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
pub use tracing::{debug, info, warn};

fn backend(tag: &str) -> (AppImageInstallable, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let core = Arc::new(AppImageBackendCore::new(
        CommandExecutor::new(false, false),
        tmp.path().join(tag),
        tmp.path().join("bin"),
        true,
        false,
        Vec::new(),
    ));
    (AppImageInstallable { core }, tmp)
}
async fn record(core: &AppImageBackendCore, name: &str, entry: AppImageState) {
    tokio::fs::create_dir_all(&core.install_dir)
        .await
        .expect("the install dir");
    core.commit_state(vec![(name.to_string(), entry)], Vec::new())
        .await
        .expect("writing the state");
}
async fn reaped() -> Reaped {
    crate::harness::reaped_for_a_test(GuardScope::Remove).await
}
async fn touch(path: &std::path::Path) {
    tokio::fs::create_dir_all(path.parent().expect("a parent"))
        .await
        .expect("the parent dir");
    tokio::fs::write(path, b"ELF").await.expect("the file");
}
mod a_download_only_appimage_has_no_path_entry_to_drop;
mod a_failed_removal_keeps_the_record_rather_than_forgetting_the_appimage;
mod an_appimage_is_listed_by_the_url_install_was_given;
mod removing_an_appimage_takes_the_file_and_the_path_entry;
mod removing_something_that_was_never_recorded_is_not_a_failure;
