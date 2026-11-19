//! The execution layer, split by what each part is responsible for.
//!
//! **`MockExecutor` is not behind `cfg(test)` and must not be.** The integration crate links the
//! library without that cfg, so a `cfg(test)` mock is invisible to the integration suite that
//! drives it. Whether it ships behind a `test-support` feature is a separate decision (#200 left
//! it open on purpose); the split changes where the code lives, not whether it is compiled.

mod command;
mod dry_run;
mod layer;
mod mock;
mod raw;

pub use command::*;
pub use dry_run::*;
pub use layer::*;
pub use mock::*;
pub use raw::*;

use crate::core::account::Account;
#[cfg(windows)]
use crate::core::launch::windows_effective_command;
use crate::core::launch::{describe, forget_path_lookups, program_exists};
use crate::core::sudo::SudoState;
use crate::core::supervise::{Stopping, LOCK_DIRS};
use crate::core::{Error, ExitPolicy, Result, Retryability};
use async_trait::async_trait;
use dashmap::DashMap;
use fs2::FileExt;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::process::Command as StdCommand;
pub use std::process::Output as StdOutput;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing::debug;
