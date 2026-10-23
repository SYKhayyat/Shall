use serde_json::Value;
pub use shall::core::Package;
pub use shall::parsers::language::*;
pub use shall::parsers::{or_unrecognised, ParseResult, Unrecognised};
pub use shall::utils::text::sanitize;

const PIP: &str = r#"[{"name": "pip", "version": "26.1.2", "latest_version": "26.2.1", "latest_filetype": "wheel"}]"#;
mod gem_reads_the_right_side_of_the_comparison;
mod nothing_outdated_is_nothing_not_a_row;
mod npm_reports_latest_rather_than_the_range_satisfying_wanted;
mod pip_reports_the_latest_version_not_the_installed_one;
