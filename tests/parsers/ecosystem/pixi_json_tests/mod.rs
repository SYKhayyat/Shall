pub use shall::core::Package;
pub use shall::parsers::ecosystem::*;
pub use shall::parsers::{or_unrecognised, ParseResult, Unrecognised};
pub use shall::utils::text::sanitize;

const REAL: &str = r#"[
  { "name": "ripgrep",
    "dependencies": [ { "name": "ripgrep", "version": "15.2.0" } ],
    "exposed": [ { "exposed_name": "rg", "executable": "rg" } ] }
]"#;
mod a_pulled_in_dependency_is_not_reported_as_installed;
mod an_empty_array_is_an_empty_machine_and_everything_else_is_unread;
mod an_environment_is_one_package_at_its_own_version;
mod the_json_and_the_tree_agree_about_the_same_machine;
mod what_an_environment_exposes_is_not_a_package;
