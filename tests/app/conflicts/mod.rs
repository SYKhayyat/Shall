pub use shall::app::conflicts::*;
pub use shall::core::PackageSpec;
use std::collections::BTreeMap;

fn spec(backend: &str, name: &str, version: Option<&str>) -> PackageSpec {
    let mut options = shall::config::grammar::Options::default();
    if let Some(v) = version {
        options.set("version", v.to_string());
    }
    PackageSpec {
        name: name.into(),
        backend: backend.into(),
        options,
        requires: vec![],
        present: true,
    }
}
mod matching_versions_across_backends_is_shadowing_not_mismatch;
mod no_conflict_for_single_backend;
mod same_backend_twice_is_not_a_conflict;
mod same_tool_two_providers_without_versions_is_shadowing;
mod version_mismatch_across_backends_is_flagged;
