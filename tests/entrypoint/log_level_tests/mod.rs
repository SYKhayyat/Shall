pub use anyhow::{Context, Result};
use clap::Parser;
pub use shall::app::entrypoint::*;
pub use shall::app::App;
pub use shall::cli::{Cli, Commands};
pub use shall::config::Config;
pub use shall::core::Output;
pub use shall::verbs::{
    check::*, cleanup::*, declare::*, ephemeral::*, history::*, inventory::*, packages::*, plan::*,
    portable::*, setup::*, sync::*, upgrade::*,
};
use std::collections::HashMap;
use std::env;
use tracing::warn;
use tracing_subscriber::EnvFilter;

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}
mod a_long_flag_is_never_read_letter_by_letter;
mod an_attached_short_value_is_not_scored_for_verbosity;
mod an_ordinary_run_says_nothing_about_itself;
mod asking_for_more_gets_more_in_both_spellings;
mod bundled_short_flags_are_read_letter_by_letter;
mod flags_stop_at_the_double_dash;
mod quiet_wins_over_loud_whichever_order_they_come_in;
mod the_commands_that_write_take_the_lock;
mod the_program_path_is_not_a_flag;
mod the_readers_are_exactly_the_commands_that_read;
