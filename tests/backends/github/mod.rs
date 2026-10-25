use async_trait::async_trait;
pub use serde::{Deserialize, Serialize};
pub use shall::backends::artifact::teardown::{still_installed, tear_down, Deployed};
pub use shall::backends::artifact::{
    self, default_formats, system_pkg, ArtifactOptions, Asset as ArtifactAsset, AssetPattern,
    Entry as ArchiveEntry, Format, FormatOrder, Platform, Request as SelectRequest,
};
pub use shall::backends::github::*;
pub use shall::core::{
    security::{generate_checksum, verify_checksum},
    ArtifactLock, LockFile,
};
pub use shall::utils::archive::extract_archive;
use std::collections::HashMap;
pub use std::path::{Path, PathBuf};
use std::sync::Arc;
pub use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
pub use tracing::{debug, info, warn};

fn release(tag: &str) -> GithubRelease {
    GithubRelease {
        version: tag.to_string(),
        assets: vec![],
    }
}
fn lock(version: &str, asset: &str, format: &str) -> ArtifactLock {
    ArtifactLock {
        version: Some(version.to_string()),
        asset: asset.to_string(),
        url: format!("https://example.invalid/{}", asset),
        format: format.to_string(),
        selected_by: None,
        sha256: Some("abc123".into()),
        ..Default::default()
    }
}
fn installed(version: &str, assets: &[&str]) -> GithubState {
    GithubState {
        repo: "sharkdp/fd".into(),
        version: version.to_string(),
        install_path: "/opt/shall/sharkdp_fd".into(),
        artifacts: assets
            .iter()
            .map(|a| InstalledArtifact {
                asset: (*a).to_string(),
                format: "tarball".into(),
                bin_path: Some(format!("/home/u/.local/bin/{}", a)),
                installed_by: None,
                system_package: None,
            })
            .collect(),
    }
}
fn tarballs() -> FormatOrder {
    FormatOrder::new(vec![Format::Tarball, Format::Binary])
}
mod a_403_that_is_not_a_rate_limit_is_never_waited_on;
mod a_deb_is_installable_only_where_dpkg_exists;
mod a_lock_without_an_install_asks_github;
mod a_pin_no_tag_answers_names_both_spellings_it_tried;
mod a_pin_that_answers_to_both_spellings_is_an_error_naming_both;
mod a_pin_that_moved_past_the_lock_asks_github;
mod a_pinned_installed_package_is_answered_without_the_network;
mod a_pinned_set_of_several_is_answered_without_the_network;
mod a_pinned_version_matches_the_tag_with_or_without_a_v;
mod a_reset_inside_the_ceiling_is_waited_out_and_retried;
mod a_reset_past_the_ceiling_is_refused_with_the_wait_it_would_have_taken;
mod a_response_that_is_not_a_403_is_left_alone;
mod an_install_that_drifted_from_the_lock_asks_github;
mod an_unpinned_line_always_asks_github;
mod changing_formats_under_a_pin_asks_github_again;
mod changing_the_asset_pattern_under_a_pin_asks_github_again;
mod either_spelling_alone_resolves_to_that_release;
mod one_of_a_locked_set_missing_from_disk_asks_github_again;
mod the_ceiling_is_the_only_thing_that_decides_between_them;
mod the_token_goes_only_to_hosts_github_serves;
mod two_assets_unpack_into_two_directories;
