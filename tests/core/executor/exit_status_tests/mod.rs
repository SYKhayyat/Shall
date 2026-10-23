pub use shall::core::executor::{fabricate_status, CommandExecutor};
pub use shall::core::{exit_policy, Error, ExitPolicy, Retryability};
pub use std::process::Output as StdOutput;

fn finished(code: i32, stdout: &str, stderr: &str) -> StdOutput {
    StdOutput {
        status: fabricate_status(code),
        stdout: stdout.as_bytes().to_vec(),
        stderr: stderr.as_bytes().to_vec(),
    }
}
fn executor_for(policy: ExitPolicy) -> CommandExecutor {
    CommandExecutor::new(true, false).with_exit_policy(policy)
}
const CHOCO_UNINSTALL_ABSENT: &str =
    include_str!("../../../../tests/fixtures/choco/uninstall-not-installed.txt");
const CHOCO_INSTALL_OK: &str = include_str!("../../../../tests/fixtures/choco/install-success.txt");
mod a_choco_failure_names_the_reason_and_not_its_troubleshooting_footer;
mod a_choco_package_that_failed_outranks_a_benign_reboot_code;
mod a_fabricated_status_round_trips_only_what_this_os_can_hold;
mod a_failed_command_shows_the_line_that_explains_it_and_not_the_bucket_update;
mod a_failure_carries_its_retryability;
mod a_manager_with_a_bare_policy_gets_the_tail_and_a_count;
mod a_policy_belongs_to_its_backend_and_not_to_a_program_name;
mod a_single_enormous_line_is_cut_rather_than_printed_whole;
mod an_executor_with_no_policy_classifies_nothing;
mod an_invisible_character_in_a_managers_output_is_named_not_drawn;
mod choco_msi_reboot_codes_are_benign_and_do_not_leak;
mod ordinary_nonzero_is_never_benign;
mod scoop_missing_manifest_is_a_failure_despite_exit_zero;
mod signal_termination_is_never_benign;
mod winget_install_of_an_absent_name_is_not_the_success_its_exit_code_claims;
mod winget_noteworthy_codes_are_benign;
