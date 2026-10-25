pub use shall::app::apply::execs::judge_script_perms;
pub use shall::app::apply::execs::*;
pub use shall::config::ExecTrust;
pub use shall::core::{Error, LockFile, Result};
pub use tracing::{debug, info, warn};

struct Fx {
    _tmp: tempfile::TempDir,
    execs: Execs<'static>,
    _config: std::sync::Arc<shall::config::Config>,
    _registry: std::sync::Arc<shall::backends::BackendRegistry>,
    _journal: std::sync::Arc<tokio::sync::Mutex<shall::core::Journal>>,
    _reaping: std::sync::Arc<shall::app::sync::guard::Reaping>,
}
fn fx() -> Fx {
    let tmp = tempfile::tempdir().unwrap();
    let config = std::sync::Arc::new(shall::config::Config::default());
    let executor = shall::core::CommandExecutor::new(true, false);
    let registry = std::sync::Arc::new(shall::backends::BackendRegistry::new());
    let journal = std::sync::Arc::new(tokio::sync::Mutex::new(
        shall::core::Journal::at(tmp.path().join("journal.jsonl")).unwrap(),
    ));
    let reaping = std::sync::Arc::new(shall::app::sync::guard::Reaping::new());
    // Leaked so the borrows outlive the struct; a test process does not care.
    let execs: Execs<'static> = Execs {
        config: Box::leak(Box::new(config.clone())),
        executor: Box::leak(Box::new(executor)),
        registry: Box::leak(Box::new(registry.clone())),
        journal: Box::leak(Box::new(journal.clone())),
        reaping: Box::leak(Box::new(reaping.clone())),
    };
    Fx {
        _tmp: tmp,
        execs,
        _config: config,
        _registry: registry,
        _journal: journal,
        _reaping: reaping,
    }
}
mod a_script_that_vanished_between_plan_and_apply_is_an_error_not_a_success;
mod the_exec_table_parses_and_defaults_to_not_world_writable;
mod the_gate_reads_real_files;
mod the_trust_levels_judge_the_same_bits_differently;
