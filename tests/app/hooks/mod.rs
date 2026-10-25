use mlua::Lua;
pub use rhai::{Engine, Scope};
pub use shall::app::hooks::*;
pub use shall::config::Config;
pub use shall::core::hook_lock::{hash_script, hook_id, refusal, HookLedger};
pub use shall::core::{Error, LockFile, Result};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command;
pub use tracing::{debug, info};

fn hooks_with(script: &str) -> LuaHooks {
    let mut config = Config::default();
    config.hooks.insert(
        "after_install".to_string(),
        HashMap::from([("pkg".to_string(), script.to_string())]),
    );
    LuaHooks::new(&config).expect("hooks")
}
fn commands() -> (&'static str, &'static str) {
    if cfg!(windows) {
        ("cmd /c exit 0", "cmd /c exit 3")
    } else {
        ("true", "exit 3")
    }
}
mod a_failing_command_in_a_rhai_hook_fails_the_hook;
mod a_hook_naming_an_absent_interpreter_says_so_plainly;
mod a_lua_hook_knows_all_four;
mod a_python_hook_runs_wherever_python_is_installed;
mod a_rhai_hook_can_reach_the_shell;
mod a_rhai_hook_indented_in_a_toml_block_still_runs;
mod a_rhai_hook_knows_all_four;
mod a_rhai_hook_reaches_the_whole_standard_library_vars_has;
mod a_shebang_hook_runs_a_process_and_is_told_which_package;
mod every_dialect_is_handed_the_same_four_facts;
mod stripping_the_marker_does_not_move_the_lines_under_it;
mod the_dialect_is_chosen_by_the_first_line;
mod the_rhai_marker_is_not_handed_to_the_engine_but_the_shebang_is;
mod the_shipped_example_config_calls_functions_that_exist;
