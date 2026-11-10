//! The review apparatus: the tests that read the tree rather than drive the product.
//!
//! **A third kind of test, and the folder is the rule.** A test that drives the `shall` binary
//! or the `Fixture` harness is an end-to-end test; a test that reads the `src/` tree and asserts
//! a property of the *code* is a review test; a test that drives `shall::`'s library APIs
//! belongs to the src file that owns it. This folder holds the second kind — the repo already
//! called that category "the review apparatus", and thirty-odd gates were doing this kind of
//! reading from the top level of `tests/`.
//!
//! **The first file here is the one that had to move for a reason.** `#116`'s fix is a policy
//! about what the mutation gate may claim, and a policy written only in a workflow file is a
//! convention: this folder is where the assertion about it belongs.

mod every_inline_test_module_is_on_an_exception_table_with_a_reason;
mod the_mutation_gate_retries_a_failed_test;

mod a_blocking_wait_is_off_the_runtime_tests;
mod a_check_that_names_an_effect_looks_for_it;
mod a_citation_in_a_comment_still_points_at_its_claim_tests;
mod a_configured_capability_is_a_registered_one_tests;
mod a_dated_essay_stays_only_while_a_living_document_cites_it;
mod a_gate_name_cited_anywhere_resolves_tests;
mod a_ledger_is_read_and_written_as_one_step_tests;
mod a_missing_kernel_module_is_not_a_coverage_regression_tests;
mod a_module_is_a_subject_not_a_pile_tests;
mod a_pinned_toolchain_would_replace_the_version_matrix_tests;
mod a_plan_installs_only_declarations_tests;
mod a_pub_fn_nobody_calls_is_dead_tests;
mod a_removal_token_cannot_be_minted_tests;
mod a_setting_the_config_declares_is_a_setting_that_acts_tests;
mod a_silenced_advisory_says_why_tests;
mod a_soft_bound_excuse_names_a_line_the_harness_prints_tests;
mod a_spawned_child_has_an_owner_tests;
mod a_string_carries_no_source_indentation_tests;
mod a_version_pin_is_honoured_or_explained_tests;
mod a_writer_that_reaches_the_disk_goes_through_one_tests;
mod an_archive_cannot_wrap_past_the_size_bound_tests;
mod backend_count_matches_the_spec_tests;
mod backend_is_data_not_code_tests;
mod deploy_refusal_precedes_the_download_tests;
mod dirty_host_fixture_isolation_tests;
mod dry_run_marker_tests;
mod every_documentation_link_resolves_tests;
mod every_example_is_checked_tests;
mod every_ruling_cited_in_code_resolves_to_one_tests;
mod every_shipped_script_is_executable_tests;
mod fanout_cap_reads_the_setting_tests;
mod grade6_binary_reachable_oracle_tests;
mod grade6_gate_parity_sees_whole_jobs_tests;
mod grader_gate_parity_tests;
mod grammar_table_matches_the_spec_tests;
mod id_namespaces_do_not_collide_tests;
mod lifecycle_coverage_union_tests;
mod named_commands_exist_tests;
mod no_comment_says_the_same_thing_twice_tests;
mod os_native_argv_coverage_tests;
mod output_is_sanitized_tests;
mod phase_is_the_sync_order_tests;
mod planner_scope_enumeration_tests;
mod priority_gates_every_fan_out_tests;
mod prompt_guard_tests;
mod removal_guard_enumeration_tests;
mod the_class_table_is_read_not_copied_tests;
mod the_composition_root_is_not_a_service_tests;
mod the_exit_table_is_generated_not_retyped_tests;
mod the_fall_through_ledger_can_be_measured;
mod the_panic_sites_are_counted_and_not_just_read_past;
mod the_planner_never_reads_the_filesystem_tests;
mod the_readme_install_promise_matches_reality_tests;
mod the_release_publishes_what_the_installers_ask_for_tests;
mod the_review_apparatus_is_rust_tests;
mod the_smoke_tier_reports_what_it_did_not_run;
mod the_stated_lifecycle_coverage_matches_the_ratchet_tests;
mod wal_enumeration_tests;
mod why_entries_are_attached_to_something_tests;
mod windows_ci_runs_nimble_canary_tests;
