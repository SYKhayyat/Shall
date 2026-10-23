use serde_json::Value;
pub use shall::core::Package;
pub use shall::parsers::language::*;
pub use shall::parsers::{or_unrecognised, ParseResult, Unrecognised};
pub use shall::utils::text::sanitize;

const REAL: &str = r#"Changed current directory to /root/.composer
{
"installed": [
    {
        "name": "psr/log",
        "direct-dependency": true,
        "version": "1.1.4",
        "latest": "3.0.2",
        "latest-status": "update-possible"
    }
]
}"#;
mod composer_reads_latest_past_its_own_banner;
mod nothing_outdated_is_nothing;
