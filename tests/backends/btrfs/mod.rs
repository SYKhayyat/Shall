use async_trait::async_trait;
use dashmap::DashMap;
pub use shall::backends::btrfs::*;
pub use shall::core::executor::{DryRunOutput, MockExecutor};
pub use shall::core::{CommandExecutor, Error, Installable, MetadataProvider, Queryable};
use std::fs;
use std::path::Path;
use std::sync::Arc;
pub use tracing::{debug, info, warn};

fn core_with(mounts: &str, responses: &[(&str, &str)]) -> BtrfsBackendCore {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    for (cmd, out) in responses {
        mock.set_response(
            cmd,
            Ok(DryRunOutput {
                stdout: out.as_bytes().to_vec(),
                stderr: vec![],
            }
            .into()),
        );
    }
    let exec = CommandExecutor::with_layer(true, false, mock, vfs, Arc::new(DashMap::new()));
    // A distinct file per test: the tests run in parallel in one process.
    let stem = format!(
        "shall-btrfs-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    );
    let f = std::env::temp_dir().join(format!("{}-mounts", stem));
    std::fs::write(&f, mounts).expect("fixture mount table");
    BtrfsBackendCore {
        executor: exec,
        name: "btrfs".to_string(),
        mounts_file: f,
        fstab_file: std::env::temp_dir().join(format!("{}-fstab", stem)),
    }
}
fn mounts(table: &str) -> Vec<BtrfsMount> {
    btrfs_mounts_in(table)
}
fn subvol_of(mounts: &[BtrfsMount], path: &str) -> Option<String> {
    subvol_arg(mounts, path).map(|(subvol, _)| subvol)
}
mod a_missing_fstab_is_empty_for_writing_and_nothing_to_drop;
mod a_mount_at_a_subvolume_reports_paths_relative_to_that_mount;
mod a_mounted_subvolume_reports_where_it_is_mounted;
mod a_path_that_is_not_under_a_btrfs_mount_has_no_fstab_entry;
mod a_plain_directory_is_not_an_installed_subvolume;
mod a_subvolume_is_listed_by_the_path_install_was_given;
mod a_subvolume_reachable_twice_is_listed_once_by_its_declared_name;
mod a_subvolume_that_cannot_be_unmounted_is_not_deleted;
mod a_table_with_no_btrfs_yields_no_mounts;
mod an_fstab_entry_names_the_subvolume_from_the_filesystem_root;
mod an_unreadable_fstab_is_refused_not_treated_as_empty;
mod every_live_mount_of_a_subvolume_is_found;
mod every_mounted_btrfs_is_asked_not_just_the_root;
mod info_reports_the_quota_and_the_mount_options_a_declaration_can_drift_from;
mod re_declaring_replaces_rather_than_doubles;
mod removing_a_subvolume_takes_its_fstab_entry_and_names_the_mount;
mod the_filesystem_uuid_is_read_out_of_the_real_report;
mod the_fstab_options_read_back_as_the_ones_that_were_declared;
mod the_qgroup_limit_is_read_by_name_and_none_is_not_a_number;
mod the_same_name_on_two_filesystems_is_two_subvolumes;
mod writing_an_entry_leaves_the_neighbours_alone;
