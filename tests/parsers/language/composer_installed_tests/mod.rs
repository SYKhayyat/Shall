use serde_json::Value;
pub use shall::core::Package;
pub use shall::parsers::language::*;
pub use shall::parsers::{or_unrecognised, ParseResult, Unrecognised};
pub use shall::utils::text::sanitize;

const REAL_GLOBAL_SHOW: &str = concat!(
    "Changed current directory to /root/.composer\n",
    r#"{"installed":["#,
    r#"{"name":"psr/log","version":"1.1.4","latest":"3.0.2"},"#,
    r#"{"name":"symfony/console","version":"6.4.0","latest":"7.0.1"}]}"#,
    "\n"
);
mod an_empty_global_install_is_empty_and_not_unrecognised;
mod the_installed_listing_is_read_from_under_the_banner;
mod the_outdated_probe_reads_the_same_output_and_reports_the_latest;
