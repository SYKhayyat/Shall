pub use shall::core::{Error, Result};
pub use shall::utils::file::*;
use std::fs;
use std::io::Write;
pub use std::path::{Component, Path, PathBuf};
use tempfile::NamedTempFile;
use tempfile::TempDir;

async fn fixture() -> (TempDir, std::path::PathBuf, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let owned = dir.path().join("artifacts");
    let bin = dir.path().join("bin");
    tokio::fs::create_dir_all(&owned).await.unwrap();
    tokio::fs::create_dir_all(&bin).await.unwrap();
    let src = owned.join("fd");
    tokio::fs::write(&src, b"#!/bin/sh\ntrue\n").await.unwrap();
    (dir, src, bin)
}
mod a_batch_repairs_a_torn_tail_once_and_not_per_line;
mod a_directory_in_the_way_is_refused_and_what_is_in_it_survives;
mod a_directory_that_cannot_be_created_says_which_one;
mod a_missing_list_is_an_empty_list_and_comments_are_not_entries;
mod an_empty_batch_does_not_touch_the_file;
mod appending_to_a_file_that_ends_cleanly_adds_no_blank_line;
mod appending_to_a_file_that_ends_mid_line_does_not_join_onto_it;
mod ends_mid_line_reads_the_last_byte_not_the_whole_file;
mod it_deploys_into_an_empty_bin_dir;
mod it_refuses_a_symlink_that_points_somewhere_else;
mod it_refuses_to_replace_a_file_shall_did_not_deploy;
mod it_replaces_a_symlink_that_points_into_its_own_artifacts;
mod it_replaces_the_path_this_backend_recorded_last_time;
mod removing_a_directory_takes_the_whole_tree;
mod removing_a_link_to_a_directory_removes_the_link_and_not_the_directory;
mod removing_something_that_is_already_gone_is_done_not_an_error;
mod replacing_a_directory_symlink_does_not_delete_what_it_pointed_at;
mod several_lines_are_appended_in_order_and_all_of_them_arrive;
mod the_async_remover_treats_a_link_the_same_way;
