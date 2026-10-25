pub use shall::app::run::*;
pub use shall::app::sandbox::{Sandbox, SandboxConfig};
pub use shall::app::sync::resolver::StateResolver;
pub use shall::app::ShimManager;
pub use shall::backends::BackendRegistry;
pub use shall::config::Config;
pub use shall::core::{Error, PackageSpec, Result};
use std::sync::Arc;
use tempfile::tempdir;
use tokio::process::Command;
pub use tracing::{debug, error, info, instrument, warn};

fn deployed_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}
mod a_command_that_names_a_path_is_left_exactly_as_written;
mod a_path_with_only_the_shim_on_it_falls_back_to_the_bare_name;
mod a_shim_on_path_is_never_what_the_runner_spawns;
