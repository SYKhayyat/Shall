pub use shall::core::Package;
pub use shall::parsers::apt::*;
pub use shall::parsers::{or_unrecognised, OutputParser, ParseResult};
pub use shall::utils::text::sanitize;

const APT: &str = "\
Listing...
gzip/noble-updates,noble-security 1.12-1ubuntu3.2 amd64 [upgradable from: 1.12-1ubuntu3.1]
libc-bin/noble-updates,noble-security 2.39-0ubuntu8.8 amd64 [upgradable from: 2.39-0ubuntu8.7]
libpam-runtime/noble-updates,noble-security 1.5.3-5ubuntu5.6 all [upgradable from: 1.5.3-5ubuntu5.5]
";
mod apt_reports_the_available_version_not_the_installed_one;
mod nothing_upgradable_is_nothing;
mod the_suite_and_the_banner_are_not_packages;
