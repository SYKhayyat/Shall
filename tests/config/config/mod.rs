pub use serde::{Deserialize, Serialize};
pub use shall::config::config::*;
pub use shall::core::{Error, Result};
pub use shall::model::Layout;
pub use shall::utils::{safe_config_dir, safe_data_dir};
use std::collections::HashMap;
use std::fs;
pub use std::path::{Path, PathBuf};

type FlagPair = (
    &'static str,
    fn(&mut CliOverrides),
    fn(&mut Config),
    fn(&Config) -> bool,
);
fn flag_pairs() -> Vec<FlagPair> {
    vec![
        (
            "dry_run",
            (|o: &mut CliOverrides| o.dry_run = true) as fn(&mut CliOverrides),
            (|c: &mut Config| c.dry_run = true) as fn(&mut Config),
            (|c: &Config| c.dry_run) as fn(&Config) -> bool,
        ),
        ("yes", |o| o.yes = true, |c| c.yes = true, |c| c.yes),
        (
            "verbose",
            |o| o.verbose = true,
            |c| c.verbose = true,
            |c| c.verbose,
        ),
        (
            "allow_mass_removal",
            |o| o.allow_mass_removal = true,
            |c| c.allow_mass_removal = true,
            |c| c.allow_mass_removal,
        ),
        (
            "allow_mass_install",
            |o| o.allow_mass_install = true,
            |c| c.allow_mass_install = true,
            |c| c.allow_mass_install,
        ),
    ]
}
mod a_flag_that_was_passed_turns_the_setting_on;
mod a_missing_file_still_remembers_where_it_looked;
mod a_named_manager_list_pins_those_managers_and_no_others;
mod a_setting_nobody_asked_for_stays_off;
mod a_setting_written_in_the_file_survives_a_flag_nobody_passed;
mod a_trailing_star_protects_by_prefix;
mod an_unparseable_freeze_list_falls_back_to_everything;
mod preferences_cannot_move_the_repo_it_lives_in;
mod protection_is_exact_not_substring;
mod sandboxed_puts_every_path_under_the_sandbox;
mod snapshot_retention_reads_the_one_config_dialect;
mod the_freeze_list_narrows_a_bare_lock_in_the_commands_own_words;
mod the_journal_table_defaults_to_what_shipped_before_it;
mod the_lock_table_defaults_to_what_shipped_before_it;
mod the_lock_table_parses_and_a_missing_key_keeps_its_default;
mod the_preferences_file_sits_inside_the_repo;
mod the_query_table_defaults_to_asking_once;
mod the_shipped_example_is_a_file_shall_would_accept;
