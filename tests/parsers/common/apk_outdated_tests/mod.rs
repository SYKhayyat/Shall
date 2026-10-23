pub use shall::core::Package;
pub use shall::parsers::common::*;
pub use shall::parsers::{or_unrecognised, ParseResult};
pub use shall::utils::text::sanitize;

const APK: &str = "\
Installed:                                Available:
apk-tools-3.0.6-r0                      < 3.0.7-r0\x20
libapk-3.0.6-r0                         < 3.0.7-r0\x20
nodejs-24.17.0-r0                       < 24.18.1-r0\x20
";
mod apk_splits_the_name_the_same_way_the_listing_does;
mod nothing_outdated_is_nothing;
mod the_header_is_not_a_package;
