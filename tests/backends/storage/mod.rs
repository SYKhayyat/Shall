use async_trait::async_trait;
use dashmap::DashMap;
pub use shall::backends::storage::*;
pub use shall::core::executor::{DryRunOutput, MockExecutor};
pub use shall::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result,
};
use std::process::Output;
use std::sync::Arc;
pub use tracing::{debug, info, warn};

fn storage_layer(responses: &[(&str, Result<Output>)]) -> (Arc<MockExecutor>, CommandExecutor) {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    for (cmd, res) in responses {
        mock.set_response(cmd, res.clone());
    }
    let exec =
        CommandExecutor::with_layer(false, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    (mock, exec)
}
fn out(text: &str) -> Result<Output> {
    Ok(DryRunOutput {
        stdout: text.as_bytes().to_vec(),
        stderr: vec![],
    }
    .into())
}
async fn removal_token() -> shall::app::sync::guard::Reaped {
    crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Sync).await
}
mod a_declared_size_grows_a_volume_and_will_not_shrink_one_unasked;
mod a_size_that_is_not_a_size_is_refused_before_anything_is_decided;
mod lvm_create_needs_a_group_size_and_volume;
mod lvm_name_splits_into_group_and_volume;
mod lvm_removal_of_an_absent_volume_is_done_not_an_error;
mod lvm_remove_confirms_and_names_the_path;
mod the_dataset_listing_reads_mountpoints_and_knows_when_there_is_none;
mod the_dataset_listing_tells_no_quota_apart_from_a_quota;
mod the_volume_listing_reads_sizes_in_bytes;
mod zfs_cannot_verify_is_refused_not_read_as_absent;
mod zfs_creates_and_destroys_by_name;
mod zfs_destroys_a_dataset_that_is_there;
mod zfs_properties_become_set_commands;
mod zfs_removal_of_an_absent_dataset_is_done_not_an_error;
