pub use shall::core::Package;
pub use shall::parsers::windows::*;
pub use shall::parsers::{or_unrecognised, ParseResult, Unrecognised};
pub use shall::utils::text::sanitize;

const CHOCO_HUMAN: &str = include_str!("../../../../tests/fixtures/choco/search-ripgrep.txt");
const CHOCO_MACHINE: &str =
    include_str!("../../../../tests/fixtures/choco/search-ripgrep-limitoutput.txt");
const CHOCO_NOT_FOUND: &str = include_str!("../../../../tests/fixtures/choco/search-not-found.txt");
mod choco_search_finding_nothing_yields_nothing;
mod choco_search_keeps_the_version;
mod choco_search_yields_packages_and_never_the_banner_or_the_summary;
