use async_trait::async_trait;
pub use shall::backends::go::*;
pub use shall::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result, Upgradable,
};
use std::path::PathBuf;
use std::sync::Arc;
pub use tracing::{info, warn};

const SUBPACKAGE: &str = include_str!("../../../tests/fixtures/go/version-m-subpackage.txt");
const MODULE_ROOT: &str = include_str!("../../../tests/fixtures/go/version-m-module-root.txt");
#[cfg(windows)]
const GOPATH_WINDOWS: &str = include_str!("../../../tests/fixtures/go/env-gopath-windows.txt");
mod a_package_at_the_module_root_still_carries_its_version;
mod a_windows_gopath_keeps_its_drive_letter;
mod an_empty_gopath_resolves_to_nothing_rather_than_to_bin;
mod binary_name_strips_path_and_version;
mod falls_back_to_path_when_no_mod;
mod install_and_upgrade_end_their_options_before_the_module;
mod non_go_binary_yields_none;
mod parses_module_and_version_from_mod_line;
mod the_first_entry_of_a_gopath_list_owns_bin;
mod the_identity_is_the_package_path_not_the_module;
mod the_module_path_goes_behind_the_terminator;
