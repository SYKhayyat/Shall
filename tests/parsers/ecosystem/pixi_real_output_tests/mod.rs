pub use shall::core::Package;
pub use shall::parsers::ecosystem::*;
pub use shall::parsers::{or_unrecognised, ParseResult, Unrecognised};
pub use shall::utils::text::sanitize;

const SEARCH: &str = include_str!("../../../../tests/fixtures/pixi/search-ripgrep.txt");
const NOT_FOUND: &str = include_str!("../../../../tests/fixtures/pixi/search-not-found.txt");
mod names_only_on_pixi_output_is_what_the_bug_looked_like;
mod no_field_label_or_separator_is_ever_a_package;
mod pixi_finding_nothing_yields_nothing;
mod pixi_search_reads_the_record_and_emits_no_junk;
