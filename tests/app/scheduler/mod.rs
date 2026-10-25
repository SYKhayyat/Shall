//! The tests that mirror `src/app/scheduler/`, module for module.

pub use shall::app::scheduler::*;
pub use shall::config::config::ScheduleConfig;
pub use shall::core::{CommandExecutor, Error};
use std::path::{Path, PathBuf};

fn cfg(cron: &str) -> ScheduleConfig {
    ScheduleConfig {
        name: "nightly".into(),
        cron: cron.into(),
        command: "sync".into(),
        notification: None,
        enabled: None,
        persistent: None,
        jitter: None,
        elevated: None,
        user: None,
    }
}

fn here() -> TaskTarget {
    TaskTarget { account: None }
}

fn me() -> TaskTarget {
    TaskTarget::resolve(Some(
        shall::core::account::Account::current().unwrap().name(),
    ))
    .unwrap()
}

fn task_xml(triggers: &str, settings_enabled: &str, run_level: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-16\"?>\n\
         <Task version=\"1.2\">\n\
         <Triggers>{triggers}</Triggers>\n\
         <Principals><Principal id=\"Author\"><UserId>S-1-5-18</UserId>{run_level}\
         </Principal></Principals>\n\
         <Settings><MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>\
         <Enabled>{settings_enabled}</Enabled></Settings>\n\
         <Actions Context=\"Author\"><Exec>\
         <Command>C:\\Program Files\\shall\\shall.exe</Command><Arguments>sync</Arguments>\
         </Exec></Actions>\n</Task>"
    )
}

mod a_boot_job_still_says_where_its_output_goes;
mod a_cron_windows_cannot_express_is_refused_rather_than_widened;
mod a_generated_file_that_exists_is_removed;
mod a_generated_file_that_is_already_gone_is_the_wanted_end_state;
mod a_named_accounts_schedule_lives_in_that_accounts_own_directory;
mod a_real_task_scheduler_document_reads_the_way_the_fixtures_say_it_does;
mod a_refusal_names_the_option_it_is_about;
mod a_removal_that_cannot_happen_is_an_error_naming_the_file;
mod a_schedule_declared_silent_is_rendered_disarmed_everywhere;
mod a_schedule_for_an_account_that_does_not_exist_is_refused;
mod a_systemd_unit_written_and_read_back_is_the_same_spec;
mod a_task_shall_created_reads_back_equal_to_what_shall_rendered;
mod a_trigger_shall_does_not_write_is_unreadable_rather_than_wrong;
mod a_trigger_shall_wrote_reads_back_as_the_when_shall_rendered;
mod an_account_with_no_user_manager_is_refused_before_anything_is_written;
mod an_unscoped_schedule_uses_what_it_always_used;
mod every_option_is_either_expressed_or_refused_by_name;
mod schtasks_maps_every_shape_a_cron_can_take;
mod schtasks_never_emits_a_time_task_scheduler_cannot_read;
mod schtasks_start_time_is_zero_padded;
mod standing_speaks_the_three_valued_answer_in_effect_wants;
mod systemd_oncalendar_mapping;
mod the_query_is_decoded_whichever_width_it_arrives_in;
mod the_rendered_when_and_the_read_when_use_one_vocabulary;
mod the_run_level_is_read_where_task_scheduler_writes_it;
mod the_same_target_drives_the_render_the_read_and_the_removal;
mod the_schedule_log_rotates_only_once_it_is_large;
mod the_task_switch_is_read_from_settings_and_not_from_a_trigger;
mod the_timer_carries_the_options_that_are_declared_and_nothing_else;
mod the_trigger_count_counts_triggers_and_not_the_block_around_them;
mod the_windows_spec_names_what_runs_when_and_at_which_privilege;
mod two_accounts_get_two_distinct_unit_paths;
mod weekdays_compare_as_a_set_not_as_a_list;
