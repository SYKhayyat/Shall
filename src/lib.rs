//! Shall - declarative configuration for a machine.
//!
//! You write what the machine should have in a file; `sync` installs what is missing
//! and removes what is no longer listed. Packages are the largest kind and not the only
//! one: repositories, services, schedules, symlinks, OS and desktop settings, scripts,
//! generated declarations, dotfile trees and firewall rules are all declared the same way.

// The function-length ratchet (#204): `clippy.toml` sets the threshold, and this makes the
// lint fire in the product crate where `cargo clippy -D warnings` (every CI leg, both release
// scripts) turns it into a build error. Every function over the threshold carries an allow
// naming the issue that will split it; a new one without one fails the build.
#![warn(clippy::too_many_lines)]

pub mod app;
pub mod backends;
pub mod cli;
pub mod config;
pub mod core;
pub mod model;
pub mod parsers;
pub mod utils;
/// The command implementations.
///
/// In the library rather than the binary so the suite can reach them. `main.rs` declared
/// `mod verbs;`, which put ~8,500 lines of real logic — the lock/unlock ledger, `check_health`,
/// the failure-attribution classifier, `reconcile` itself — where no test binary could link to
/// it, so it could only be exercised by spawning the program.
pub mod verbs;

// Primary entry points for library consumers
pub use app::App;
pub use config::Config;
pub use core::error::{Error, Result};
pub use core::manager::BackendCore;
pub use core::transaction::TransactionConfig;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
