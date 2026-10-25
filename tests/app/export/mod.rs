pub use shall::app::export::*;
pub use shall::core::{Error, Result};
pub use std::path::{Path, PathBuf};

fn p(backend: &str, name: &str, ver: Option<&str>) -> Pkg {
    (backend.into(), name.into(), ver.map(|s| s.into()))
}
mod beside_keeps_the_extension_readable;
mod brewfile_lists_only_brew_packages;
mod empty_when_no_package_applies;
mod format_parse_accepts_aliases;
mod free_path_steps_past_a_fallback_that_is_also_taken;
mod no_nix_packages_means_no_nix_file;
mod package_json_is_valid_and_scoped_to_node;
mod requirements_pins_versions_and_falls_back;
mod the_nix_format_takes_both_prefixes_and_renders_a_module;
