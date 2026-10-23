pub use shall::core::Package;
pub use shall::parsers::common::*;
pub use shall::parsers::{or_unrecognised, ParseResult};
pub use shall::utils::text::sanitize;

const BREW: &str = r#"{"formulae":[
    {"name":"jq","installed_versions":["1.7"],"current_version":"1.7.1"},
    {"name":"ripgrep","installed_versions":["14.1.0"],"current_version":"15.2.0"}],
  "casks":[{"name":"firefox","installed_versions":"140.0","current_version":"141.0"}]}"#;
mod a_banner_before_the_json_does_not_hide_the_answer;
mod brew_reports_the_current_version_and_includes_casks;
mod nothing_outdated_is_nothing;
