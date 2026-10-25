pub use shall::config::config::ScheduleConfig;
pub use shall::config::grammar::statement::SCHEDULE_OPTION_KEYS as KNOWN_KEYS;
pub use shall::config::grammar::{GrammarError, Options, Origin, Result};
pub use shall::model::schedule::*;

fn opts(pairs: &[(&str, &str)]) -> Options {
    let mut o = Options::default();
    for (k, v) in pairs {
        o.insert(*k, *v);
    }
    o
}
fn origin() -> Origin {
    Origin::new("schedules", 3)
}
fn shipped() -> Vec<String> {
    shall::config::Config::default().guard.never_unattended
}
mod a_bad_cron_is_refused_at_parse_time;
mod a_complete_line_maps_to_a_schedule_config;
mod a_jitter_is_a_duration_and_a_bounded_one;
mod a_missing_cron_is_an_error_that_names_the_line;
mod a_missing_run_is_an_error;
mod a_name_that_prefixes_another_is_not_a_match;
mod a_timer_may_not_run_rebuild_or_purge_undeclared_out_of_the_box;
mod add_line_writes_a_block_and_refuses_a_duplicate_name;
mod an_empty_cron_value_is_treated_as_missing;
mod an_empty_list_refuses_nothing;
mod an_unknown_key_is_refused_not_ignored;
mod cron_accepts_five_fields_six_fields_and_macros_and_refuses_garbage;
mod every_spelling_of_yes_and_no_reads_and_anything_else_is_refused;
mod notify_is_optional;
mod remove_line_takes_the_whole_block_and_leaves_its_neighbours;
mod taking_a_name_out_of_the_list_permits_the_command;
mod the_refusal_names_the_list_it_came_from;
mod the_refusal_reads_the_command_not_the_whole_line;
