//! **One test binary.**
//!
//! `tests/` held 101 files and cargo auto-discovers each as its own target, so every one of them
//! was fat-LTO-linked against a 100k-line crate under `codegen-units = 1` — and 36 of them never
//! call the library API at all, only spawning `CARGO_BIN_EXE_shall`. The `target/` directory that
//! produced reached **194 GB** and filled a 944 GB disk mid-build.
//!
//! **The suite already paid this cost once and wrote it down.** `mock_providers/mod.rs` opens by
//! recording that a top-level `mock_providers.rs` became *"a 716 KB binary containing zero
//! tests… compiled nineteen times"*, and that the fix was to make it a directory. This is the
//! same fix at the scale of the whole directory: one target, ninety-nine link units saved, and
//! `mock_providers` compiled once rather than nineteen times.
//!
//! **The conversion deleted nothing.** `autotests = false` in `Cargo.toml` is what stops cargo
//! claiming each file as a target, and the list below is what replaces the claim. A file added
//! to `tests/` and not listed here does not run — the one cost of this arrangement, and why
//! `every_test_file_is_in_the_suite` at the bottom fails when the two disagree.
//!
//! Four files were later renamed into the directory's filename-as-a-sentence convention, out of
//! the pre-v7 `test_*_wiring` style they were the last of; one test was deleted, by name, in
//! favour of the successor whose own header cites it as the gap it exists to close.
//!
//! Two modules carry a `cfg`: their files opened with an inner `#![cfg(...)]`, which cannot
//! survive becoming a module. On the `mod` line it is strictly better — off-platform the module
//! is not compiled at all rather than compiled to nothing.

#![allow(unused_imports)]

/// The shared test doubles. Declared once here, which is the whole point: nineteen files said
/// `mod mock_providers;` and each got its own copy.
mod a_drifted_ecosystem_does_not_strand_the_config_tests;
mod a_removal_token_cannot_be_minted_tests;
mod an_archive_cannot_wrap_past_the_size_bound_tests;

mod mock_providers;

/// The `Fixture` sixteen files wrote out by hand, three of them differently — see the module's
/// own header for what the three ways were and why the union is the correct one.
mod harness;

/// The four assertions every scanning gate makes about its own exemption table. Nine files
/// wrote them out by hand and three of the nine were missing the one that matters.
mod ledger;

/// **The mirror.** `src/app/sync/guard.rs` owns `tests/app/sync/guard/`, and the rule is that a
/// src file's tests live in a folder of its own — so that "where is the test for this?" has an
/// answer before you have to ask it. `src/app/sync/guard.rs` was 3,473 lines, 1,944 of them
/// tests, and over the recorded ceiling because of them; the code is 1,530 lines and the ceiling
/// row is retired rather than raised.
mod app;
mod entrypoint;

/// **The mirror, continued.** `src/core/sudo.rs` owns `tests/core/sudo/`, under the same rule
/// as `src/app/sync/guard.rs`: a src file's tests live in a folder of their own, so that "where
/// is the test for this?" has an answer before you have to ask it.
mod backends;
mod config;
mod core;
mod parsers;
mod verbs;

/// **The review apparatus.** The tests that read the tree rather than drive the product — the
/// gates that assert a property of the *code*, and the policies written in a workflow file
/// rather than in a function. `#120` names this folder; this is the first file to land in it.
mod review;

mod a_backend_is_a_row_tests;
mod a_batch_of_installs_is_one_command_tests;
mod a_blocking_wait_is_off_the_runtime_tests;
mod a_citation_in_a_comment_still_points_at_its_claim_tests;
mod a_config_travels_between_machines_tests;
mod a_configured_capability_is_a_registered_one_tests;
mod a_downloaded_artifact_is_named_by_its_key_tests;
mod a_failed_sync_fails_under_every_flag_tests;
mod a_fan_out_of_managers_is_asked_concurrently_tests;
mod a_fed_child_is_drained_while_it_is_fed_tests;
mod a_finished_package_is_recorded_in_batches_tests;
mod a_firewall_teardown_is_a_removal_tests;
mod a_gate_name_cited_anywhere_resolves_tests;
mod a_ledger_is_read_and_written_as_one_step_tests;
mod a_lister_cannot_report_what_was_removed_tests;
mod a_machine_converges_tests;
mod a_missing_kernel_module_is_not_a_coverage_regression_tests;
mod a_module_is_a_subject_not_a_pile_tests;
mod a_mutation_forgets_only_what_it_could_have_changed_tests;
mod a_parser_can_say_it_did_not_understand_tests;
mod a_pinned_toolchain_would_replace_the_version_matrix_tests;
mod a_plan_installs_only_declarations_tests;
mod a_plan_reaps_only_what_it_was_asked_about_tests;
mod a_pub_fn_nobody_calls_is_dead_tests;
mod a_reader_writes_nothing_tests;
mod a_rehearsal_asks_the_guard_the_act_asks_tests;
mod a_setting_the_config_declares_is_a_setting_that_acts_tests;
mod a_shim_runs_what_its_line_named_tests;
mod a_silenced_advisory_says_why_tests;
mod a_skip_does_not_erase_the_rest_of_the_drift_tests;
mod a_skipped_install_is_not_an_undeclared_leftover_tests;
mod a_spawned_child_has_an_owner_tests;
mod a_string_carries_no_source_indentation_tests;
mod a_version_pin_is_honoured_or_explained_tests;
mod a_writer_that_reaches_the_disk_goes_through_one_tests;
mod absent_marker_coverage_tests;
mod adapter_tables_share_one_mechanism_tests;
mod an_ephemeral_shell_leaves_nothing_behind_tests;
mod an_exemption_table_is_audited_the_same_way_tests;
mod an_extension_surface_has_a_front_door_tests;
mod an_option_after_a_value_is_not_swallowed_tests;
mod an_option_list_survives_the_seam_tests;
mod an_orphan_of_a_killed_run_is_taken_back_tests;
mod an_unanswered_command_is_a_defect_tests;
mod ansi_is_for_terminals_tests;
mod argv_drift_tests;
mod automation_lifecycle_tests;
mod backend_count_matches_the_spec_tests;
mod backend_is_data_not_code_tests;
mod backend_tests;
mod benign_exit_contradiction_tests;
mod byte_order_mark_tests;
mod conda_is_a_data_row_tests;
mod config_root_is_absolute_tests;
mod critical_paths_tests;
mod declared_options_converge_tests;
mod deploy_refusal_precedes_the_download_tests;
mod dirty_host_fixture_isolation_tests;
mod dotfiles_tree_is_a_pile_of_links_tests;
mod dry_run_every_verb_tests;
mod dry_run_marker_tests;
mod dry_run_tests;
mod every_documentation_link_resolves_tests;
mod every_example_is_checked_tests;
mod every_ruling_cited_in_code_resolves_to_one_tests;
mod every_shipped_script_is_executable_tests;
mod exec_lifecycle_tests;
mod exit_code_contract_tests;
mod failure_class_line_tests;
mod fanout_cap_reads_the_setting_tests;
mod feature_logic_tests;
mod first_hour_tests;
mod grade2_check_extras_tests;
mod grade2_flag_drift_blindspot_tests;
mod grade2_info_tests;
mod grade2_plan_extras_tests;
mod grade2_wedged_manifest_tests;
mod grade3_dry_run_data_dir_tests;
mod grade3_pixi_list_fixture_tests;
mod grade3_plan_guard_kind_tests;
mod grade3_protected_inspector_tests;
mod grade3_resource_idempotency_tests;
mod grade4_adopt_respects_the_manifest_tests;
mod grade4_heal_reports_its_own_failure_tests;
mod grade4_keyword_is_not_a_package_tests;
mod grade4_refusal_names_the_line_tests;
mod grade5_adopt_takes_services_tests;
mod grade6_backslash_is_not_set_math_tests;
mod grade6_binary_reachable_oracle_tests;
mod grade6_gate_parity_sees_whole_jobs_tests;
mod grade6_option_edit_reaches_the_machine_tests;
mod grade7_protected_skip_is_reported_tests;
mod grader_dry_run_siblings_tests;
mod grader_extras_guard_tests;
mod grader_gate_parity_tests;
mod grader_refusal_exit_code_tests;
#[cfg(windows)]
mod grader_shim_exit_code_tests;
mod grader_transient_claim_tests;
mod grader_unknown_backend_tests;
mod grammar_table_matches_the_spec_tests;
mod hardening_tests;
mod helm_verify_flag_tests;
mod help_map_tests;
mod hook_reentrancy_tests;
mod id_namespaces_do_not_collide_tests;
mod installed_listing_fixture_tests;
mod json_output_is_a_document_tests;
mod latency_budget_tests;
mod ledger_file_rules_tests;
mod lifecycle_coverage_union_tests;
mod lock_default_tests;
mod lock_scope_tests;
mod lock_unlock_scope_tests;
mod named_commands_exist_tests;
mod no_comment_says_the_same_thing_twice_tests;
mod one_parser_reads_a_removal_target_tests;
mod option_table_coverage_tests;
mod os_native_argv_coverage_tests;
mod output_is_sanitized_tests;
mod parser_fixture_tests;
mod phase_is_the_sync_order_tests;
mod planner_scope_enumeration_tests;
mod priority_gates_every_fan_out_tests;
mod prompt_guard_tests;
#[cfg(target_os = "linux")]
mod pty_tests;
mod recovery_finishes_what_it_can_tests;
mod removal_guard_enumeration_tests;
mod resource_plan_family_tests;
mod security_and_resiliency_tests;
mod snapshot_restore_reaches_every_provider_tests;
mod startup_budget_tests;
mod sync_and_check_agree_about_convergence_tests;
mod terminator_probe_tests;
mod the_class_table_is_read_not_copied_tests;
mod the_composition_root_is_not_a_service_tests;
mod the_engine_runs_the_graph_in_order_tests;
mod the_engine_sends_what_the_manager_can_take_tests;
mod the_exit_table_is_generated_not_retyped_tests;
mod the_kernel_assembles_what_it_was_configured_with_tests;
mod the_log_covers_what_cannot_be_recomputed_tests;
mod the_other_fan_outs_fan_out_too_tests;
mod the_planner_never_reads_the_filesystem_tests;
mod the_probe_argv_does_not_depend_on_the_package_tests;
mod the_readme_install_promise_matches_reality_tests;
mod the_review_apparatus_is_rust_tests;
mod the_rollup_counts_the_same_machine_as_the_detail_view_tests;
mod the_scheduler_hands_out_a_generation_at_a_time_tests;
mod the_stated_lifecycle_coverage_matches_the_ratchet_tests;
mod time_travel_tests;
mod two_accounts_are_two_resources_tests;
mod unknown_backend_family_tests;
mod verbs_are_reachable_tests;
mod wal_enumeration_tests;
mod why_entries_are_attached_to_something_tests;
mod windows_ci_runs_nimble_canary_tests;

/// **A file in `tests/` that is not a module here never runs.**
///
/// That is the price of `autotests = false`, and it is payable only because it is checked. A
/// suite that silently stops running a file is worse than one that takes ninety-nine link units.
///
/// **The scan is recursive, because the tree is.** The mirror puts a src file's tests in a
/// folder of their own — `src/app/sync/guard.rs` owns `tests/app/sync/guard/` — and a flat scan
/// of the top level would have called that folder's contents invisible, which is the exact
/// failure this test exists to prevent, one directory down. So every `.rs` file under `tests/`
/// is checked against the module file that owns the directory it sits in, and every directory is
/// checked against the module file that owns its parent.
#[test]
fn every_test_file_is_in_the_suite() {
    fn declared_in(path: &std::path::Path) -> std::collections::BTreeSet<String> {
        std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()))
            .lines()
            .filter_map(|l| {
                let l = l.trim();
                let l = l
                    .strip_prefix("pub mod ")
                    .or_else(|| l.strip_prefix("mod "))?;
                l.strip_suffix(';').map(str::to_string)
            })
            .collect()
    }

    /// The file that *is* a directory's module: the `mod.rs` inside it, or a sibling file named
    /// after the directory.
    fn module_file_of(dir: &std::path::Path) -> std::path::PathBuf {
        let mod_rs = dir.join("mod.rs");
        if mod_rs.is_file() {
            return mod_rs;
        }
        let name = dir
            .file_name()
            .expect("a directory has a name")
            .to_string_lossy();
        let sibling = dir.with_file_name(format!("{name}.rs"));
        if sibling.is_file() {
            return sibling;
        }
        mod_rs
    }

    /// **A directory with no Rust in it is data, not a module.** `tests/fixtures/` holds 47
    /// captured command outputs and no source, and requiring a `mod` declaration for it would be
    /// a declaration of nothing. The test is the other way round: a directory that *does* hold
    /// Rust is a module, and its files are its subject.
    fn holds_rust(dir: &std::path::Path) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "rs") {
                return true;
            }
            if path.is_dir() && holds_rust(&path) {
                return true;
            }
        }
        false
    }

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut missing: Vec<String> = Vec::new();

    // The root is `main.rs`, and it owns everything directly in `tests/`.
    let root_declared = declared_in(&root.join("main.rs"));
    for entry in std::fs::read_dir(&root)
        .expect("tests/ is readable")
        .flatten()
    {
        let path = entry.path();
        let name = path
            .file_name()
            .expect("an entry has a name")
            .to_string_lossy()
            .into_owned();
        if name == "main.rs" {
            continue;
        }
        if path.is_file() {
            let stem = name.strip_suffix(".rs").expect("a .rs file has a stem");
            if !root_declared.contains(stem) {
                missing.push(format!("tests/{name} is not a module of the suite"));
            }
        } else if path.is_dir() {
            // A directory is a module of its parent — unless it holds no Rust at all, which
            // makes it data.
            if holds_rust(&path) && !root_declared.contains(&name) {
                missing.push(format!("tests/{name}/ is not a module of the suite"));
            }
        }
    }

    // Every directory below the root, and every file in it.
    let mut dirs = vec![root.clone()];
    while let Some(dir) = dirs.pop() {
        // The root's own module file is the suite's root, `main.rs`; every other directory's is
        // its `mod.rs` or the file named after it.
        let owner = if dir == root {
            dir.join("main.rs")
        } else {
            module_file_of(&dir)
        };
        let declared = if owner.is_file() {
            declared_in(&owner)
        } else {
            std::collections::BTreeSet::new()
        };
        for entry in std::fs::read_dir(&dir)
            .expect("a directory is readable")
            .flatten()
        {
            let path = entry.path();
            let name = path
                .file_name()
                .expect("an entry has a name")
                .to_string_lossy()
                .into_owned();
            if path.is_dir() {
                if holds_rust(&path) && !declared.contains(&name) {
                    missing.push(format!(
                        "{} is not a module of {}",
                        path.strip_prefix(&root).expect("under tests/").display(),
                        owner.strip_prefix(&root).expect("under tests/").display()
                    ));
                }
                dirs.push(path);
            } else if path.is_file() && name.ends_with(".rs") {
                let stem = name.strip_suffix(".rs").expect("a .rs file has a stem");
                if owner.file_name().map(|n| n == "mod.rs").unwrap_or(false)
                    || name == "mod.rs"
                    || (dir == root && name == "main.rs")
                {
                    // The directory's own module file.
                    continue;
                }
                if !declared.contains(stem) {
                    missing.push(format!(
                        "{} is not a module of {}",
                        path.strip_prefix(&root).expect("under tests/").display(),
                        owner.strip_prefix(&root).expect("under tests/").display()
                    ));
                }
            }
        }
    }

    assert!(
        missing.is_empty(),
        "these files are in tests/ and are not modules of the suite, so nothing runs them: \
         {missing:#?}\nAdd `mod <name>;` to the module file that owns them."
    );

    // And the other direction: a `mod` naming a file that is gone does not compile, so the only
    // thing left to check is that the scan found the suite at all.
    assert!(
        root_declared.len() > 90,
        "only {} module(s) found in tests/main.rs; this scan has stopped matching it",
        root_declared.len()
    );
}
