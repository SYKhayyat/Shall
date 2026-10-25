use async_trait::async_trait;
use serde_json::Value;
pub use shall::backends::nix::*;
pub use shall::core::{
    BackendCore, CommandExecutor, Error, Installable, MetadataProvider, Package, PackageSpec,
    Queryable, Result, Searchable, Upgradable,
};
use std::sync::Arc;
use tracing::info;
mod a_batch_of_installables_is_one_nix_call;
mod nix_flake_refs_and_queries_come_after_the_terminator;
mod nix_profile_list_reads_the_v3_map_keyed_by_name;
mod nix_search_empty_is_ok;
mod nix_search_parses_json_map;
