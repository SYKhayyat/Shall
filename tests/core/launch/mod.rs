//! The tests that mirror `src/core/launch.rs`.

#[cfg(windows)]
pub use shall::core::launch::windows_shim_wrap;

#[cfg(windows)]
mod a_cmd_shim_is_spawned_as_itself_never_through_a_hand_built_cmd_c;
#[cfg(windows)]
mod a_crafted_package_name_stays_one_argument_through_a_cmd_shim;
#[cfg(windows)]
mod a_powershell_level_failure_is_not_reported_as_success;
mod leaves_exe_alone;
#[cfg(windows)]
mod ps1_args_are_single_quote_escaped_no_injection;
#[cfg(windows)]
mod the_program_name_is_quoted_and_invoked_through_the_call_operator;
#[cfg(windows)]
mod wraps_ps1_via_command_with_out_string;

#[cfg(windows)]
mod grader_shim_exit_code_tests;
