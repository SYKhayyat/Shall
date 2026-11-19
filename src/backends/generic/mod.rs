//! The data-driven backend, split by capability.
//!
//! **A move only.** `backends::generic::X` names the same items it always did; what changed is
//! that `impl Installable`, `impl Queryable` and the rest each have a file, so "where is the
//! remove path" has an answer other than scrolling 2,469 lines.

mod config;
mod core;
mod install;
mod query;
mod search;
mod types;

pub use config::*;
pub use core::*;
pub use install::*;
pub use query::*;
pub use search::*;
pub use types::*;

use crate::backends::capability;
use crate::core::{
    BackendCore, CommandExecutor, Enumerable, Error, Installable, MetadataProvider, Package,
    PackageSpec, Queryable, RepoManager, Result, Searchable, Upgradable,
};
use crate::parsers::OutputParser;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};
