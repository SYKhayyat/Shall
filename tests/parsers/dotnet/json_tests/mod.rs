pub use shall::core::Package;
pub use shall::parsers::dotnet::*;
pub use shall::parsers::{or_unrecognised, ParseResult};
pub use shall::utils::text::sanitize;

const REAL: &str = r#"{"version":1,"data":[{"packageId":"dotnetsay","version":"3.0.3","commands":["dotnetsay"]}]}"#;
mod a_tool_is_its_package_id_and_version;
mod an_empty_data_array_is_an_empty_machine_and_everything_else_is_unread;
mod the_json_and_the_table_agree_about_the_same_machine;
