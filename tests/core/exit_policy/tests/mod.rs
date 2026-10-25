pub use shall::core::error::Retryability;
pub use shall::core::exit_policy::*;

const NIMBLE_BUILD_FAILED: &str =
    include_str!("../../../../tests/fixtures/nimble/install-build-failed.txt");
const NIMBLE_NOT_FOUND: &str =
    include_str!("../../../../tests/fixtures/nimble/install-not-found.txt");
const NIMBLE_UNINSTALL_MISSING: &str =
    include_str!("../../../../tests/fixtures/nimble/uninstall-not-installed.txt");
const NIMBLE_LIST: &str = include_str!("../../../../tests/fixtures/nimble/list-installed.txt");
const SCOOP_UNINSTALL_MISSING: &str =
    include_str!("../../../../tests/fixtures/scoop/uninstall-not-installed.txt");
const HELM_UNVERIFIABLE: &str =
    include_str!("../../../../tests/fixtures/helm/plugin-install-unverifiable-source.txt");
const LUAROCKS_MANIFEST_UNREACHABLE: &str =
    include_str!("../../../../tests/fixtures/luarocks/install-manifest-unreachable.txt");
const CHOCO_SOURCE_UNREACHABLE: &str =
    include_str!("../../../../tests/fixtures/choco/install-source-unreachable.txt");
const CHOCO_ABSENT_NAME: &str =
    include_str!("../../../../tests/fixtures/choco/install-absent-name.txt");
mod a_held_lock_is_transient_and_a_missing_name_is_not;
mod a_luarocks_manifest_that_would_not_download_is_transient_not_a_missing_rock;
mod a_luarocks_summary_without_a_download_failure_is_not_retried;
mod a_name_the_source_does_not_carry_is_still_absent;
mod a_permanent_marker_still_outranks_a_transient_one;
mod a_policy_with_no_markers_classifies_nothing;
mod a_prefix_marker_anchors_to_a_line_and_does_not_match_prose;
mod a_signal_kill_is_never_benign;
mod a_successful_nimble_listing_is_not_a_failure;
mod an_unreachable_source_is_not_a_missing_package;
mod benign_codes_are_per_backend_and_do_not_leak;
mod cabal_names_an_absent_package_in_both_versions_phrasings;
mod cabal_reads_an_unverifiable_hackage_root_as_the_index_and_not_as_the_package;
mod cabal_withdraws_nothing_when_the_index_failure_and_the_name_appear_together;
mod each_distro_policy_tells_a_held_lock_from_a_missing_name;
mod helm_losing_the_network_is_worth_another_attempt;
mod helm_refusing_an_unsignable_source_is_permanent_not_transient;
mod nimble_failures_are_seen_though_every_one_of_them_exits_zero;
mod no_manager_calls_a_name_absent_while_it_is_also_reporting_a_transient_failure;
mod permanent_wins_over_transient;
mod scoop_reports_a_missing_manifest_as_a_failure_and_never_retries_it;
mod scoop_sees_a_failure_that_is_not_a_missing_manifest;
mod transient_wins_over_absent;
