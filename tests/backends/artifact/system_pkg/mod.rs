pub use shall::backends::artifact::format::Format;
pub use shall::backends::artifact::system_pkg::*;
pub use shall::core::{Error, Result};
use std::path::Path;
use std::path::PathBuf;

fn deb() -> PathBuf {
    PathBuf::from("/tmp/fd_10.2.0_amd64.deb")
}
fn rpm() -> PathBuf {
    PathBuf::from("/tmp/fd-10.2.0.x86_64.rpm")
}
mod deb_install_upgrades_in_place;
mod install_refuses_a_format_it_does_not_hand_off;
mod name_query_reads_from_the_file;
mod only_deb_and_rpm_are_handoffs;
mod remove_uses_the_package_name_not_the_file;
mod rpm_install_is_upgrade_or_install_not_bare_i;
