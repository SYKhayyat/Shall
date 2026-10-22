use super::*;

pub use shall::config::grammar::statement::*;
fn known(name: &str) -> bool {
    matches!(
        name,
        // `pip` is here for `@system` (`Q49`), which is legal on exactly one backend: a
        // stand-in registry that omits it could only ever assert the refusal half.
        "apt" | "cargo" | "web" | "github" | "snap" | "flatpak" | "appimage" | "helm" | "pip"
    )
}
fn p(line: &str) -> Result<Statement> {
    parse(&Origin::new("modules/dev.txt", 3), line, &known)
}
fn options_of(line: &str) -> Options {
    match p(line).unwrap() {
        Statement::Package(d) => d.options,
        other => panic!("expected a package, got {:?}", other),
    }
}

mod a_bare_name_defers_the_capability_check_to_the_resolver;
mod a_checksum_beside_a_list_of_formats_is_not;
mod a_checksum_beside_one_pinned_format_is_legal;
mod a_checksum_needs_the_line_to_pin_one_format;
mod a_checksum_on_a_backend_that_selects_nothing_is_untouched;
mod a_helm_plugin_carries_its_install_source_and_nothing_else_may;
mod a_repeated_formats_key_is_an_ordered_list;
mod a_repeated_single_value_option_is_refused_rather_than_resolved_by_order;
mod allow_http_did_not_follow_unverified_onto_helm;
mod an_asset_pattern_is_validated_at_parse_time;
mod an_unknown_format_names_the_legal_set;
mod asset_all_and_a_checksum_cannot_both_be_set;
mod bin_names_the_executable_inside_an_archive;
mod channel_is_read_on_snap_and_flatpak;
mod channel_on_a_backend_without_version_streams_is_an_error;
mod download_only_is_read_on_a_download_backend_and_refused_elsewhere;
mod formats_is_read_on_a_backend_that_offers_a_choice;
mod formats_on_a_backend_that_decided_already_is_an_error;
mod formats_on_appimage_is_a_contradiction_and_is_refused;
mod system_is_legal_on_pip_and_refused_by_name_elsewhere;
mod there_is_no_fallback_across_channels;
mod unverified_is_legal_on_every_backend_that_verifies_something;
mod unverified_on_a_manager_with_its_own_signed_index_is_still_refused;
