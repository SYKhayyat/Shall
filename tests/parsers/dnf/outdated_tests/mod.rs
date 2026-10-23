pub use shall::core::Package;
pub use shall::parsers::dnf::*;
pub use shall::parsers::{or_unrecognised, ParseResult};
pub use shall::utils::text::sanitize;

const DNF: &str = "\
Upgrades
audit-libs.x86_64                  4.2.1-1.fc44     updates
coreutils.x86_64                   9.10-5.fc44      updates
curl.x86_64                        8.18.0-8.fc44    updates
";
const ZYPPER: &str = "\
S  | Repository                 | Name               | Current Version | Available Version | Arch
---+----------------------------+--------------------+-----------------+-------------------+-------
v  | repo-oss                   | curl               | 8.17.0-1.1      | 8.18.0-1.1        | x86_64
v  | repo-oss                   | glibc              | 2.42-1.1        | 2.43-1.1          | x86_64
";
mod a_dnf_section_banner_is_not_a_package;
mod a_zypper_header_row_is_not_a_package;
mod dnf_strips_the_arch_so_the_name_matches_the_listing;
mod nothing_outdated_is_nothing;
mod zypper_reads_available_and_not_current;
