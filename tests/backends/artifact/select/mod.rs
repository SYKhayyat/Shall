pub use shall::backends::artifact::format::{Format, FormatOrder};
pub use shall::backends::artifact::pattern::AssetPattern;
pub use shall::backends::artifact::platform::Platform;
pub use shall::backends::artifact::select::*;
use std::fmt;

fn linux64() -> Platform {
    Platform::new("linux", "x86_64")
}
fn assets(names: &[&str]) -> Vec<Asset> {
    names
        .iter()
        .map(|n| Asset::new(*n, format!("https://example.test/{}", n)))
        .collect()
}
fn request<'a>(
    formats: &'a FormatOrder,
    platform: &'a Platform,
    pattern: Option<&'a AssetPattern>,
) -> Request<'a> {
    Request {
        package: "sharkdp/fd",
        release: "v10.2.0",
        platform,
        formats,
        pattern,
    }
}
mod a_detected_order_yields_to_the_asset_that_names_the_machine;
mod a_foreign_asset_is_never_chosen_even_as_a_last_resort;
mod a_later_format_is_a_fallback_not_an_addition;
mod a_pattern_narrows_but_does_not_select;
mod a_release_with_no_assets_says_so_rather_than_listing_nothing;
mod a_user_written_format_order_wins_over_a_more_specific_asset;
mod a_version_number_in_the_name_is_not_an_extension;
mod an_asset_pattern_narrows_to_the_variant;
mod an_explicit_target_beats_a_shorter_silent_one;
mod an_extensionless_asset_is_a_binary;
mod asset_all_installs_every_match_instead_of_choosing;
mod checksums_and_signatures_are_not_offered_as_choices;
mod no_match_reports_what_the_release_actually_offered;
mod release_notes_are_not_mistaken_for_an_executable;
mod selection_does_not_depend_on_the_order_the_api_returned;
mod the_d3_case_shortest_filename_wins_and_says_so;
mod the_selection_reason_names_the_winning_rule;
