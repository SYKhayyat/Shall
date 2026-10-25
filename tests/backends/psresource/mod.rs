use async_trait::async_trait;
pub use shall::backends::psresource::*;
pub use shall::config::Config;
pub use shall::core::{
    BackendCapabilities, BackendCore, CommandExecutor, Error, Installable, MetadataProvider,
    Package, PackageSpec, Queryable, Result, Searchable, Upgradable,
};
pub use shall::parsers::common::parse_simple_list;
use std::sync::Arc;
use tracing::info;
mod validate_name_accepts_module_names;
mod validate_name_rejects_injection;
mod validate_query_allows_wildcards;
mod validate_version_accepts_semver_like;
