use once_cell::sync::OnceCell;
pub use shall::backends::node_registry::*;
pub use shall::core::{Error, Package, Result};
use std::time::Duration;
mod npm_registry_empty_objects;
mod parses_npm_registry_objects;
