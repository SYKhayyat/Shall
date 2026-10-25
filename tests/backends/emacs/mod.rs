use async_trait::async_trait;
use dashmap::DashMap;
pub use shall::backends::emacs::*;
pub use shall::core::executor::MockExecutor;
pub use shall::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result, Searchable, Upgradable,
};
use std::sync::Arc;
use tracing::info;

fn wired() -> (Arc<EmacsBackendCore>, Arc<MockExecutor>) {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let exec =
        CommandExecutor::with_layer(false, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    (Arc::new(EmacsBackendCore::new(exec)), mock)
}
mod a_batch_of_packages_is_one_emacs_and_one_archive_refresh;
mod a_batch_of_removals_is_one_emacs;
mod one_illegal_name_refuses_the_whole_batch_before_anything_runs;
