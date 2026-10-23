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
use std::collections::HashSet;
use std::env;
use tracing::warn;
use tracing_subscriber::EnvFilter;

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}
fn verbs(pairs: &[(&str, &[&str])]) -> HashMap<String, Vec<String>> {
    pairs
        .iter()
        .map(|(k, steps)| (k.to_string(), steps.iter().map(|s| s.to_string()).collect()))
        .collect()
}
fn builtins() -> HashSet<String> {
    ["sync", "upgrade", "check"]
        .into_iter()
        .map(String::from)
        .collect()
}
mod a_global_flag_on_a_step_is_refused_and_pointed_at_the_leading_form;
mod a_malformed_step_is_refused_by_name_instead_of_a_usage_exit;
mod a_name_that_is_neither_builtin_nor_verb_is_left_alone;
mod a_subcommands_own_flag_still_works_on_a_step;
mod a_verb_expands_to_one_argv_per_step;
mod a_verb_inherits_leading_global_flags_on_every_step;
mod a_verb_never_shadows_a_builtin;
mod a_verb_step_must_be_a_builtin;
mod a_verb_takes_no_arguments;
mod builtin_subcommand_is_never_shadowed;
mod every_posture_spelling_is_refused_wherever_it_hides_on_a_step;
mod expands_a_defined_alias_into_tokens;
mod expands_alias_after_a_value_taking_global_flag;
mod expands_alias_after_a_valueless_global_flag;
mod leaves_unknown_and_flag_first_tokens_alone;
mod subcommand_index_skips_flags_and_their_values;
mod the_posture_table_covers_every_global_flag_clap_defines;
mod value_flags_are_exactly_what_clap_says_take_a_value;
