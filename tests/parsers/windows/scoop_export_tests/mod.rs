pub use shall::core::Package;
pub use shall::parsers::windows::*;
pub use shall::parsers::{or_unrecognised, ParseResult, Unrecognised};
pub use shall::utils::text::sanitize;

const EXPORT: &str = r#"{
    "buckets": [ { "Name": "main", "Source": "https://github.com/ScoopInstaller/Main.git" } ],
    "apps": [
        { "Info": "", "Source": "main", "Name": "7zip",    "Version": "26.00" },
        { "Info": "Install failed", "Source": "main", "Name": "jq", "Version": "" },
        { "Info": "", "Source": "main", "Name": "halfway", "Version": "" },
        { "Info": "", "Source": "main", "Name": "fd",      "Version": "10.4.2" }
    ]
}"#;
mod a_failed_install_is_not_an_installed_package;
mod an_app_with_no_version_is_not_an_installed_package;
mod an_appless_export_is_an_empty_machine_and_a_malformed_one_is_not;
mod an_export_yields_the_installed_apps;
mod the_export_and_the_table_agree_about_the_same_machine;
